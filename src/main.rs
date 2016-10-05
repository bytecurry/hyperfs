extern crate hyper;

use std::io;
use std::io::Write;
use std::fs::File;
use std::path::Path;

use std::net::{Ipv4Addr, SocketAddrV4};

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::header::ContentLength;
use hyper::net::Fresh;
use hyper::uri::RequestUri::AbsolutePath;
use hyper::status::StatusCode::NotFound;

macro_rules! println_err {
    ($fmt: expr, $($arg:tt)*) => {
        match writeln!(&mut std::io::stderr(), $fmt,  $($arg)*) {
            Ok(_) => {},
            Err(_) => {}
        }
    };
}

fn handle_request(req: Request, resp: Response<Fresh>) {
    let result = match req.uri {
        AbsolutePath(abs_path) => if let Ok(file) = File::open(extract_path(&abs_path)) {
            file_response(file, resp)
        } else {
            not_found(resp)
        },
        _ => not_found(resp)
    };
    match result {
        Err(e) => println_err!("Error: {}", e),
        _ => {}
    };
}

fn file_response(mut file: File, mut resp: Response<Fresh>) -> io::Result<()> {
    let metadata = try!(file.metadata());
    resp.headers_mut().set(ContentLength(metadata.len()));
    resp.start().and_then(|mut res| {
        try!(io::copy(&mut file, &mut res));
        res.end()
    })
}

fn not_found(mut resp: Response<Fresh>) -> io::Result<()> {
    *resp.status_mut() = NotFound;
    resp.send(b"<h1>Not Found</h1>")
}

fn extract_path(uri: &String) -> &Path {
    let first_question = uri.find('?').unwrap_or(uri.len());
    let slice = &uri[1 .. first_question];
    Path::new(slice)
}

fn main() {
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let port = std::env::args().nth(1).and_then(|p| p.parse::<u16>().ok()).unwrap_or(3000);

    let addr = SocketAddrV4::new(ip, port);

    match Server::http(addr) {
        Ok(server) => {
            println!("Server is listening at http://{}", addr);
            server.handle(handle_request).unwrap();
        },
        Err(error) => {
            println_err!("Unable to start server: {}", error);
            std::process::exit(1);
        }
    }
}
