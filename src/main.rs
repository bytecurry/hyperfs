extern crate bytes;
extern crate futures;
extern crate http;
extern crate hyper;
extern crate tokio;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use futures::Future;
use hyper::rt;
use hyper::service::service_fn;
use hyper::StatusCode;
use hyper::{Body, Chunk, Request, Response, Server};
use tokio::codec::{Decoder, FramedRead};
use tokio::fs::File;
use tokio::io;

fn handle_request(req: Request<Body>) -> impl Future<Item = Response<Body>, Error = http::Error> {
    File::open(extract_path(&req))
        .map(file_response)
        .or_else(|_| status_response(StatusCode::NOT_FOUND))
}

fn file_response(file: File) -> Response<Body> {
    let stream = FramedRead::new(file, ChunkDecoder);
    Response::new(Body::wrap_stream(stream))
}

fn status_response(status: StatusCode) -> http::Result<Response<Body>> {
    Response::builder().status(status).body(Body::empty())
}

fn extract_path(req: &Request<Body>) -> PathBuf {
    // Remove the first character
    PathBuf::from(req.uri().path().trim_left_matches('/'))
}

struct ChunkDecoder;

impl Decoder for ChunkDecoder {
    type Item = Chunk;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut bytes::BytesMut) -> Result<Option<Chunk>, io::Error> {
        let len = buf.len();
        if len > 0 {
            Ok(Some(buf.take().freeze().into()))
        } else {
            Ok(None)
        }
    }
}

fn main() {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = std::env::args()
        .nth(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::new(ip, port);

    let server = Server::bind(&addr)
        .serve(|| service_fn(handle_request))
        .map_err(|e| {
            eprintln!("Unable to start server: {}", e);
        });

    println!("Server is listening at http://{}", addr);

    rt::run(server);
}
