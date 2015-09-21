extern crate hyper;

use std::io;
use std::fs::File;
use std::path::Path;

use std::net::{Ipv4Addr, SocketAddrV4};

use hyper::Server;
use hyper::server::Request;
use hyper::server::Response;
use hyper::header::ContentLength;
use hyper::net::Fresh;
use hyper::uri::RequestUri::AbsolutePath;

fn handle_request(req: Request, mut resp: Response<Fresh>) {
    if let AbsolutePath(abs_path) = req.uri {
        if let Ok(file) = File::open(extract_path(&abs_path)) {
            file_response(file, resp).unwrap();
        } else {
            *resp.status_mut() = hyper::NotFound;
            resp.send(b"<h1>Not Found</h1>").unwrap();
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
            panic!("Unable to start server: {}", error);
        }
    }
}
