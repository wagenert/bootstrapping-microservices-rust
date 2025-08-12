use axum::{Router, body::Body, extract::State, response::IntoResponse, routing::get};
use std::env;

#[derive(Clone)]
struct AppState {
    video_storage_host: String,
    video_storage_port: String,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").expect("PORT environment variable not set");
    let video_storage_host =
        env::var("VIDEO_STORAGE_HOST").expect("VIDEO_STORAGE_HOST environment variable not set");
    let video_storage_port =
        env::var("VIDEO_STORAGE_PORT").expect("VIDEO_STORAGE_PORT environment variable not set");

    let app_state = AppState {
        video_storage_host,
        video_storage_port,
    };
    let app = app(app_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    println!("Server running at {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/video", get(get_video))
        .with_state(state)
}

async fn get_video(State(state): State<AppState>) -> impl IntoResponse {
    let video_storage_host = &state.video_storage_host;
    let video_storage_port = &state.video_storage_port;
    let forward_response = reqwest::Client::new()
        .get(format!("http://{video_storage_host}:{video_storage_port}/video?path=SampleVideo_1280x720_1mb.mp4"))
        .send()
        .await
        .expect("Failed to forward request");
    let status_code = forward_response.status();
    let headers = forward_response.headers().clone();
    let video_data = forward_response.bytes_stream();
    (
        status_code,
        (headers, Body::from_stream(video_data)).into_response(),
    )
        .into_response()
    /*    axum::response::Response::builder()
    .status(status_code)
    .header("Content-Type", "video/mp4")
    .body(Body::from_stream(video_data))
    .unwrap()
    */
}
