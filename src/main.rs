use std::ffi::OsStr;
use std::os::unix::prelude::OsStrExt;
use std::path::PathBuf;

use axum::http::{HeaderMap, StatusCode};
use axum::routing::put;
use axum::Router;

#[tokio::main]
async fn main() {
    // our router
    let app = Router::new().route("/rename", put(rename));

    axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn rename(headers: HeaderMap) -> StatusCode {
    eprintln!("/rename");
    // Read env vars
    let src_dir = match std::env::var_os("SRC_DIR") {
        Some(d) => PathBuf::from(d),
        None => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    let dst_dir = match std::env::var_os("DST_DIR") {
        Some(d) => PathBuf::from(d),
        None => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    // Parse headers
    let src = match headers.get::<String>("X-TEMP".into()) {
        Some(v) => v,
        None => return StatusCode::BAD_REQUEST,
    };
    let dst = match headers.get::<String>("X-FILE".into()) {
        Some(v) => v,
        None => return StatusCode::BAD_REQUEST,
    };

    // Construct read src and dst
    let src = PathBuf::from(OsStr::from_bytes(src.as_bytes()));
    let src = match src.file_name() {
        Some(f) => PathBuf::from(src_dir.join(f)),
        None => return StatusCode::BAD_REQUEST,
    };
    let dst = PathBuf::from(OsStr::from_bytes(dst.as_bytes()));
    let dst = match dst.file_name() {
        Some(f) => PathBuf::from(dst_dir.join(f)),
        None => return StatusCode::BAD_REQUEST,
    };
    dbg!(&src, &dst);

    std::fs::copy(&src, &dst).unwrap();

    StatusCode::OK
}
