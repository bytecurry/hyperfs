use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

use futures_util::{stream::MapOk, TryStreamExt};
use http_body_util::{Either, Empty, StreamBody};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use tokio::fs::File;
use tokio::net::TcpListener;
use tokio_util::io::ReaderStream;

type Frame = hyper::body::Frame<Bytes>;
type Body = Either<StreamBody<MapOk<ReaderStream<File>, fn(Bytes) -> Frame>>, Empty<Bytes>>;

async fn handle_request(req: Request<Incoming>) -> http::Result<Response<Body>> {
    match File::open(extract_path(&req)).await {
        Ok(f) => Ok(file_response(f)),
        Err(_) => status_response(StatusCode::NOT_FOUND),
    }
}

fn file_response(file: File) -> Response<Body> {
    let stream = ReaderStream::new(file);
    Response::new(Body::Left(StreamBody::new(stream.map_ok(Frame::data))))
}

fn status_response(status: StatusCode) -> http::Result<Response<Body>> {
    Response::builder()
        .status(status)
        .body(Body::Right(Empty::new()))
}

fn extract_path(req: &Request<Incoming>) -> PathBuf {
    // Remove the first character
    PathBuf::from(req.uri().path().trim_start_matches('/'))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let port = std::env::args()
        .nth(1)
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::new(ip, port);

    let svc = service_fn(handle_request);

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Unable to bind to 127.0.0.1:{}", port);
            return Err(e.into());
        }
    };

    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;

        // TODO: gracefully handle SIGINT?
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            let builder = auto::Builder::new(TokioExecutor::new());
            if let Err(err) = builder.serve_connection(io, svc).await {
                eprintln!("ERROR: {:?}", err);
            }
        });
    }
}
