mod requests;

use std::env;
use axum::{routing::get, routing::post, Extension, Router};
use dotenvy::dotenv;
use requests::handle_upload;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use sqlx::postgres::PgPoolOptions;
use supabase::Client;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let endpoint = env::var("SUPABASE_ENDPOINT").expect("SUPABASE-ENDPOINT must be set");
    let service_key = env::var("SERVICE_KEY").expect("SERVICE_KEY must be set");
    
    tracing_subscriber::fmt()
        .with_target(false)
        .compact() // Use .json() here for production!
        .init();
    
    let supabase_client = Client::new(&endpoint, &service_key).expect("Failed to authenticate with supabase");
    let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await.unwrap();
    tracing::info!("Database connection established");

    // Create the axum router
    let app = Router::new()
        .route("/scrape", post(handle_upload))
        .route("/hello-world", get(hello_world))
        .layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new()
                .level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(tower_http::LatencyUnit::Micros)))
        .layer(Extension(supabase_client))
        .with_state(pool);
    
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