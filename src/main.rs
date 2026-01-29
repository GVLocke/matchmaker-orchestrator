mod requests;
mod service;

use std::env;
use std::sync::Arc;
use tokio::sync::Semaphore;
use axum::{routing::get, routing::post, Router};
use dotenvy::dotenv;
use requests::{handle_single_upload, handle_batch_upload};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use sqlx::postgres::PgPoolOptions;
use aws_sdk_s3::Client as S3Client;
use aws_config::Region;
use sqlx::PgPool;
use serde_json::Value;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub s3_client: S3Client,
    pub http_client: reqwest::Client,
    pub openai_api_key: String,
    pub resume_schema: Value,
    pub semaphore: Arc<Semaphore>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let endpoint = env::var("SUPABASE_ENDPOINT").expect("SUPABASE_ENDPOINT must be set");
    let service_key = env::var("SERVICE_KEY").expect("SERVICE_KEY must be set");
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let max_concurrent_tasks = env::var("MAX_CONCURRENT_TASKS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<usize>()
        .expect("MAX_CONCURRENT_TASKS must be a number");

    // Extract project ref and build S3 endpoint
    let (s3_endpoint, project_ref) = if endpoint.contains("127.0.0.1") || endpoint.contains("localhost") {
        let clean_endpoint = endpoint.trim_end_matches('/');
        (format!("{}/storage/v1/s3/", clean_endpoint), "local-stub".to_string())
    } else {
        let ref_id = endpoint
            .replace("https://", "")
            .replace("http://", "")
            .replace(".supabase.co", "")
            .trim_end_matches('/')
            .to_string();
        (format!("https://{}.supabase.co/storage/v1/s3/", ref_id), ref_id)
    };
    
    tracing::info!("Configured S3 Endpoint: {}", s3_endpoint);
    tracing::info!("Project Ref: {}", project_ref);

    // Load and parse schema once
    let raw_schema_string = include_str!("resume_schema.json");
    let resume_schema: Value = serde_json::from_str(raw_schema_string).expect("Invalid JSON Schema File");
    
    tracing_subscriber::fmt()
        .with_target(false)
        .compact() // Use .json() here for production!
        .init();
    
    let s3_access_key = env::var("S3_ACCESS_KEY").unwrap_or_else(|_| project_ref.clone());
    let s3_secret_key = env::var("S3_SECRET_KEY").unwrap_or_else(|_| service_key.clone());

    // Configure AWS SDK for Supabase S3
    let credentials = aws_sdk_s3::config::Credentials::new(
        s3_access_key,
        s3_secret_key,
        None,
        None,
        "supabase-storage",
    );

    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(Region::new("us-east-1")) // Region is required but ignored by Supabase
        .endpoint_url(&s3_endpoint)
        .credentials_provider(credentials)
        .load()
        .await;

    let s3_config = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .build();

    let s3_client = S3Client::from_conf(s3_config);

    let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await.unwrap();
    let http_client = reqwest::Client::new();
    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));
    
    tracing::info!("Database connection established");

    let app_state = AppState {
        pool,
        s3_client,
        http_client,
        openai_api_key,
        resume_schema,
        semaphore,
    };

    // Create the axum router
    let app = Router::new()
        .route("/ingest/interns/individual", post(handle_single_upload))
        .route("/ingest/interns/batch", post(handle_batch_upload))
        .route("/hello-world", get(hello_world))
        .layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new()
                .level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(tower_http::LatencyUnit::Micros)))
        .with_state(app_state);
    
    // Define the IP and port listener (TCP)
    let address = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    // Call axum serve to launch the web server
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    tracing::info!("hello-world handler accessed");
    "Hello, World!"
}