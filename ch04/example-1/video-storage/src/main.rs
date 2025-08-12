use axum::{
    Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use azure_core::http::StatusCode;
use azure_identity::ClientSecretCredential;
use azure_storage_blob::{
    BlobClient, BlobClientOptions,
    models::{BlobClientDownloadOptions, BlobClientGetPropertiesOptions},
};
use serde::Deserialize;
use std::{env, error::Error};
use std::{result::Result, sync::Arc};

#[derive(Deserialize)]
struct VideoName {
    name: String,
}

#[derive(Clone)]
struct AppState {
    blob_server: Arc<BlobClient>,
}

impl AppState {
    fn new(azure: BlobClient) -> Self {
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
    let tenant_id =
        env::var("TENANT_ID").expect("TENANT_ID variable not set");
    let client_id =
        env::var("CLIENT_ID").expect("CLIENT_ID variable not set");
    let client_secret_string =
        env::var("CLIENT_SECRET").expect("CLIENT_SECRET variable not set");

    let client_secret = azure_core::credentials::Secret::new(client_secret_string);
    let azure_blob_service =
        create_blob_service(storage_account_name, tenant_id, client_id, client_secret).expect("Can not create BLOB service");

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
fn create_blob_service(storage_account: String, tenant_id: String, client_id: String, client_secret: azure_core::credentials::Secret) -> Result<BlobClient, Box<dyn Error>> {
    //let credentials = DefaultAzureCredential::new()?;
    let credentials = ClientSecretCredential::new(tenant_id, client_id, client_secret, None)?;
    let blob_client = BlobClient::new(
        format!("https://{storage_account}.blob.core.windows.net/").as_str(), // endpoint
        "videos".to_string(),                                                 // container name
        "SampleVideo_1280x720_1mb.mp4".to_string(),                           // blob name
        credentials.clone(),                                                  // credential
        Some(BlobClientOptions::default()),                                   // BlobClient options
    )?;
    Ok(blob_client)
}

async fn get_video(
    State(state): State<AppState>,
    Query(_vid_name): Query<VideoName>,
) -> impl IntoResponse {
    println!("Retrieving properties");
    let props = state
        .blob_server
        .get_properties(Some(BlobClientGetPropertiesOptions::default()))
        .await
        .unwrap();
    match props.status() {
        StatusCode::Ok => {
            // Access properties directly from props.body
            let properties = props.into_raw_body();
            // You can now use `properties` as needed
        }
        _ => panic!("Request for properties failed!"),
    }
    let blob = state
        .blob_server
        .download(Some(BlobClientDownloadOptions::default()))
        .await
        .unwrap();
    match blob.status() {
        StatusCode::Ok => {
            let bytes = blob.into_raw_body();
        }
        _ => panic!("Request for blob failed!"),
    }
}
