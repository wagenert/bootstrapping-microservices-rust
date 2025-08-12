use axum::{
    Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use azure_core::http::StatusCode;
use azure_core::http::headers::HeaderName;
use azure_identity::ClientSecretCredential;
use azure_storage_blob::{
    BlobContainerClient, BlobContainerClientOptions,
    models::{BlobClientDownloadOptions, BlobClientGetPropertiesOptions},
};
use serde::Deserialize;
use std::{env, error::Error};
use std::{result::Result, sync::Arc};

#[derive(Deserialize)]
struct VideoName {
    path: String,
}

#[derive(Clone)]
struct AppState {
    blob_server: Arc<BlobContainerClient>,
}

impl AppState {
    fn new(azure: BlobContainerClient) -> Self {
        Self {
            blob_server: Arc::new(azure),
        }
    }
}

#[tokio::main]
async fn main() {
    // Retrieve environment variables
    let port = env::var("PORT").expect("PORT environment variable not set");
    let storage_account_name =
        env::var("STORAGE_ACCOUNT_NAME").expect("STORAGE_ACCOUNT_NAME variable not set");
    let tenant_id = env::var("TENANT_ID").expect("TENANT_ID variable not set");
    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID variable not set");
    let client_secret_string = env::var("CLIENT_SECRET").expect("CLIENT_SECRET variable not set");

    // Create secret from access token
    let client_secret = azure_core::credentials::Secret::new(client_secret_string);

    // Create BlobContainerClient
    let azure_blob_service =
        create_blob_service(storage_account_name, tenant_id, client_id, client_secret)
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
        .route("/video", get(get_video))
        .with_state(state)
}
fn create_blob_service(
    storage_account: String,
    tenant_id: String,
    client_id: String,
    client_secret: azure_core::credentials::Secret,
) -> Result<BlobContainerClient, Box<dyn Error>> {
    //let credentials = DefaultAzureCredential::new()?;
    let credentials =
        ClientSecretCredential::new(tenant_id.as_str(), client_id, client_secret, None)?;
    let blob_client = BlobContainerClient::new(
        format!("https://{storage_account}.blob.core.windows.net/").as_str(), // endpoint
        "videos".to_string(),                                                 // container name
        credentials.clone(),                                                  // credential
        Some(BlobContainerClientOptions::default()),                          // BlobClient options
    )?;
    Ok(blob_client)
}

async fn get_video(
    State(state): State<AppState>,
    Query(vid_name): Query<VideoName>,
) -> impl IntoResponse {
    let video_path = vid_name.path;
    let container_server = state.blob_server.clone();
    let blob_client = container_server.blob_client(video_path);
    println!("Retrieving properties");
    let props = blob_client
        .get_properties(Some(BlobClientGetPropertiesOptions::default()))
        .await
        .unwrap();

    let blob_properties = match props.status() {
        StatusCode::Ok => {
            // Access properties directly from props.body
            props.headers()
            // You can now use `properties` as needed
        }
        _ => panic!("Request for properties failed!"),
    };

    println!("Retrieving blob");
    let blob = blob_client
        .download(Some(BlobClientDownloadOptions::default()))
        .await
        .unwrap();
    let stream = match blob.status() {
        StatusCode::Ok => blob.into_raw_body(),
        _ => panic!("Request for blob failed!"),
    };

    println!("Extracting headers");
    // Headers are lower-cased
    let content_type = blob_properties
        .get_str(&HeaderName::from_static("content-type"))
        .unwrap_or("application/octet-stream");
    let content_length = blob_properties
        .get_as::<u32, _>(&HeaderName::from_static("content-length"))
        .unwrap_or(0);
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", content_type)
        .header("Content-Length", content_length)
        .body(axum::body::Body::from_stream(stream))
        .unwrap()
}
