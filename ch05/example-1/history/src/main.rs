use axum::{Router, routing::get};
use std::env;

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
    Router::new().route("/", get(|| async { "Hello, World!" }))
}
