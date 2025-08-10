use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::get,
};
use object_store::azure::{MicrosoftAzure, MicrosoftAzureBuilder};
use serde::Deserialize;
use std::env;
use std::{result::Result, sync::Arc};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[derive(Deserialize)]
struct VideoName(String);

#[derive(Debug, Clone)]
struct AppState {
    blob_server: Arc<MicrosoftAzure>,
}

impl AppState {
    fn new(azure: MicrosoftAzure) -> Self {
        Self {
            blob_server: Arc::new(azure),
        }
    }
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").expect("PORT environment variable not set");
    let storage_account_name =
        env::var("STORAGE_ACCOUNT_NAME").expect("STORAGE_ACCOUNT_NAME variable not set");
    let storage_access_key =
        env::var("STORAGE_ACCESS_KEY").expect("STORAGE_ACCESS_KEY variable not set");

    let azure_blob_service = create_blob_service(storage_account_name, storage_access_key)
        .expect("Can not create BLOB service");

    let app_state = AppState::new(azure_blob_service);

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
        .with_state(state)
        .route("/video", get(get_video))
}
fn create_blob_service(
    storage_account: String,
    storage_key: String,
) -> Result<MicrosoftAzure, object_store::Error> {
    MicrosoftAzureBuilder::new()
        .with_account(storage_account)
        .with_access_key(storage_key)
        .with_container_name("videos")
        .build()
}

async fn get_video(
    State(state): State<Arc<AppState>>,
    Query(vid_name): Query<VideoName>,
) -> impl IntoResponse {
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
