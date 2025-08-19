use axum::{
    Router,
    body::Body,
    response::IntoResponse,
    routing::{get, post},
};
use std::env;

#[tokio::main]
async fn main() {
    let port = env::var("PORT").expect("PORT environment variable not set");
    // Extremely important comment
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
        .route("/viewed", post(handle_viewed_request))
}

async fn handle_viewed_request() -> impl IntoResponse {
    println!("Received viewed message");
    (axum::http::StatusCode::OK, Body::from("")).into_response()
}
