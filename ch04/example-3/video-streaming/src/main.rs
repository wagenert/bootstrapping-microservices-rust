use axum::{
    Router,
    body::Body,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use mongodb::bson::doc;
use serde::Deserialize;
use std::{env, str::FromStr};

#[derive(Deserialize)]
struct VideoId {
    id: String,
}

#[derive(Deserialize)]
struct Video {
    #[serde(rename = "videoPath")]
    video_path: String,
}

#[derive(Clone)]
struct AppState {
    video_storage_host: String,
    video_storage_port: String,
    client: mongodb::Client,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").expect("PORT environment variable not set");
    let video_storage_host =
        env::var("VIDEO_STORAGE_HOST").expect("VIDEO_STORAGE_HOST environment variable not set");
    let video_storage_port =
        env::var("VIDEO_STORAGE_PORT").expect("VIDEO_STORAGE_PORT environment variable not set");
    let db_host = env::var("DBHOST").expect("DBHOST environment variable not set");
    let db_name = env::var("DBNAME").expect("DBNAME environment variable not set");

    let connection_string = format!("mongodb://{db_host}/{db_name}");
    let mut client_options = mongodb::options::ClientOptions::parse(connection_string)
        .await
        .expect("Can not create connection options");
    let server_api = mongodb::options::ServerApi::builder()
        .version(mongodb::options::ServerApiVersion::V1)
        .build();
    client_options.server_api = Some(server_api);
    let client = mongodb::Client::with_options(client_options).expect("Can not create clients");
    client.database("video-streaming");
    let app_state = AppState {
        video_storage_host,
        video_storage_port,
        client,
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

async fn get_video(
    State(state): State<AppState>,
    Query(video_id): Query<VideoId>,
) -> impl IntoResponse {
    let video_id =
        mongodb::bson::oid::ObjectId::from_str(&video_id.id).expect("Invalid video ID format");
    println!("Successfully created video id: {video_id}");
    let videos = state
        .client
        .database("video-streaming")
        .collection::<Video>("videos");
    println!("Successfully connected to the videos collection");
    let video_record = videos.find_one(doc! { "_id": &video_id });
    match video_record.await {
        Ok(Some(video)) => {
            let video_path = video.video_path;
            let video_storage_host = &state.video_storage_host;
            let video_storage_port = &state.video_storage_port;
            let forward_response = reqwest::Client::new()
                .get(format!(
                    "http://{video_storage_host}:{video_storage_port}/video?path={video_path}"
                ))
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
        }
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "Video not found").into_response(),
        Err(e) => {
            eprintln!("Error fetching video: {}", e);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            )
                .into_response()
        }
    }
}
