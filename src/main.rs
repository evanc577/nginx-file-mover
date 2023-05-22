use std::ffi::OsStr;
use std::os::unix::prelude::OsStrExt;
use std::path::PathBuf;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::put;
use axum::Router;

#[derive(Clone)]
struct AppState {
    src_dir: PathBuf,
    dst_dir: PathBuf,
}

enum RenameError {
    Header(String),
    Move,
}

impl IntoResponse for RenameError {
    fn into_response(self) -> axum::response::Response {
        match self {
            RenameError::Header(h) => (StatusCode::BAD_REQUEST, format!("invalid header {}", h)),
            RenameError::Move => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to move file".into(),
            ),
        }
        .into_response()
    }
}

#[tokio::main]
async fn main() {
    // Read env vars
    let src_dir = PathBuf::from(std::env::var_os("SRC_DIR").unwrap());
    let dst_dir = PathBuf::from(std::env::var_os("DST_DIR").unwrap());
    let state = AppState { src_dir, dst_dir };

    // Construct server socket
    let socket = std::env::args().nth(1).unwrap().parse().unwrap();

    // Router
    let app = Router::new()
        .route("/rename", put(rename))
        .with_state(state);

    axum::Server::bind(&socket)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn rename(headers: HeaderMap, State(state): State<AppState>) -> Result<(), RenameError> {
    // Join directory from `dir` with file name from `header`
    let make_path = |dir: PathBuf, header: &str| {
        headers
            .get::<String>(header.into())
            .map(|v| PathBuf::from(OsStr::from_bytes(v.as_bytes())))
            .map(|f| dir.join(f))
            .ok_or_else(|| RenameError::Header(header.to_owned()))
    };
    let src = make_path(state.src_dir, "X-TEMP")?;
    let dst = make_path(state.dst_dir, "X-FILE")?;

    // Move file
    println!("/rename moving {src:?} to {dst:?}");
    if let Err(_) = std::fs::rename(&src, &dst) {
        // If rename fails, copy it instead. nginx `client_body_in_file_only clean` will delete the
        // temp file
        std::fs::copy(&src, &dst).map_err(|_| RenameError::Move)?;
    }

    Ok(())
}
