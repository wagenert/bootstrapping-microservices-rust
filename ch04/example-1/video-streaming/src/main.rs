use axum::{
    Router,
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::get,
};
use std::env;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[tokio::main]
async fn main() {
    let port = env::var("PORT").expect("PORT environment variable not set");

    let app = app();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    println!("Server running at {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn app() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/video", get(get_video))
}

async fn get_video() -> impl IntoResponse {
    let file_path = "video/SampleVideo_1280x720_1mb.mp4";
    println!("Serving video from: {file_path}");
    match File::open(&file_path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let mut headers = HeaderMap::new();
            headers.insert("Content-Type", HeaderValue::from_static("video/mp4"));
            axum::response::Response::builder()
                .status(axum::http::StatusCode::OK)
                .header("Content-Type", "video/mp4")
                .body(axum::body::Body::from_stream(stream))
                .unwrap()
        }
        Err(err) => (
            axum::http::StatusCode::NOT_FOUND,
            format!("File not found: {err}"),
        )
            .into_response(),
    }
}
