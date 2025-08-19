use axum::{
    Router,
    body::Body,
    extract::{Json, State},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Serialize)]
struct VideoPath {
    video_path: String,
}

#[derive(Clone)]
struct AppState {
    history_collection: mongodb::Collection<VideoPath>,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT").expect("PORT environment variable not set");
    let db_host = env::var("DBHOST").expect("DBHOST environment variable not set");
    let db_name = env::var("DBNAME").expect("DBNAME environment variable not set");

    let mut client_options = mongodb::options::ClientOptions::parse(db_host)
        .await
        .expect("Can not create connection options");
    let server_api = mongodb::options::ServerApi::builder()
        .version(mongodb::options::ServerApiVersion::V1)
        .build();
    client_options.server_api = Some(server_api);
    let client = mongodb::Client::with_options(client_options).expect("Can not create clients");
    let db = client.database(&db_name);
    let collection = db.collection::<VideoPath>("history");

    let state = AppState {
        history_collection: collection,
    };

    // Extremely important comment
    let app = app(state);

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
        .route("/viewed", post(handle_viewed_request))
        .with_state(state)
}

async fn handle_viewed_request(
    State(app_state): State<AppState>,
    Json(payload): Json<VideoPath>,
) -> impl IntoResponse {
    println!(
        "Received viewed message with video path: {}",
        payload.video_path
    );
    match app_state.history_collection.insert_one(payload).await {
        Ok(_) => (axum::http::StatusCode::OK, Body::from("")).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Body::from(format!("{e}")),
        )
            .into_response(),
    }
}
