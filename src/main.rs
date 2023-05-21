#![deny(warnings)]
use std::net::SocketAddr;

use bytes::Bytes;
use hyper::server::conn::http1;
use http_body_util::Full;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, Result, StatusCode,};
use tokio::net::TcpListener;

use clap::Parser;

use once_cell::sync::Lazy;

static ARGS: Lazy<Args> = Lazy::new(get_args);

fn get_args() -> Args {
    Args::parse()
}

/// A HTTP servers which serves one file and one file only, no matter the path
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Object to be served
    #[arg(short, long)]
    object: String,

    /// Filename of the object when served
    #[arg(short, long)]
    filename: String,

    /// Socket Address to bind to
    #[arg(short, long, default_value = "0.0.0.0:3000")]
    address: String,

    /// Error message to display when the file is not present
    #[arg(short, long, default_value = "NOT FOUND")]
    error_msg: String,
}

async fn response(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/")
        | (&Method::GET, "/index.html")
        | (&Method::GET, "/index.htm") => simple_file_send(ARGS.object.as_str()).await,
        _ => simple_file_send(ARGS.object.as_str()).await,
    }
}

async fn simple_file_send(filename: &str) -> Result<Response<Full<Bytes>>> {
    if let Ok(contents) = tokio::fs::read(filename).await {
        let body = contents.into();
        return Ok(
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .header("Content-Disposition", format!("{}{}{}", "attachment; filename=\"", ARGS.filename.as_str(), "\""))
                .body(Full::new(body))
                .unwrap()
        );
    }

    //404 NOT FOUND
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "text/plain")
        .body(Full::new(Bytes::from(ARGS.error_msg.as_bytes())))
        .unwrap())
}

#[tokio::main]
pub async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let addr: SocketAddr = ARGS.address
                                .parse()
                                .expect("Unable to parse socket address");

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(response))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}