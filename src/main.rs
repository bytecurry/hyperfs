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

macro_rules! println_err {
    ($fmt: expr, $($arg:tt)*) => {
        match writeln!(&mut std::io::stderr(), $fmt,  $($arg)*) {
            Ok(_) => {},
            Err(_) => {}
        }
    };
}

macro_rules! try_return {
    ($e: expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => {
                println_err!("Error: {}", e);
                return;
            }
        }
    }}
}

fn handle_request(req: Request, mut resp: Response<Fresh>) {
    if let AbsolutePath(abs_path) = req.uri {
        if let Ok(file) = File::open(extract_path(&abs_path)) {
            try_return!(file_response(file, resp));
        } else {
            *resp.status_mut() = hyper::NotFound;
            try_return!(resp.send(b"<h1>Not Found</h1>"));
        }
    }
}

fn file_response(mut file: File, mut resp: Response<Fresh>) -> io::Result<u64> {
    let metadata = try!(file.metadata());
    resp.headers_mut().set(ContentLength(metadata.len()));
    resp.start().and_then(|mut res| io::copy(&mut file, &mut res))
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
