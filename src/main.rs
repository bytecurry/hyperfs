use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use futures_util::stream::TryStreamExt;
use http::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::StatusCode;
use hyper::{Body, Error, Request, Response, Server};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

async fn handle_request(req: Request<Body>) -> Result<Response<Body>> {
    match File::open(extract_path(&req)).await {
        Ok(f) => file_response(f),
        Err(_) => status_response(StatusCode::NOT_FOUND),
    }
}

fn file_response(file: File) -> Result<Response<Body>> {
    let stream = FramedRead::new(file, BytesCodec::new()).map_ok(|b| b.freeze());
    Ok(Response::new(Body::wrap_stream(stream)))
}

fn status_response(status: StatusCode) -> Result<Response<Body>> {
    Response::builder().status(status).body(Body::empty())
}

fn extract_path(req: &Request<Body>) -> PathBuf {
    // Remove the first character
    PathBuf::from(req.uri().path().trim_start_matches('/'))
}

#[tokio::main]
async fn main() {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = std::env::args()
        .nth(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::new(ip, port);

    let make_svc = make_service_fn(|_| async { Ok::<_, Error>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Unable to start server: {}", e);
    }
}
