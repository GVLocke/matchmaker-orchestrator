mod requests;
use axum::{routing::get, routing::post, Json, Router};
use requests::handle_upload;
use std::net::SocketAddr;
use tower_http::trace::{self, MakeSpan, TraceLayer};
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact() // Use .json() here for production!
        .init();

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
                .latency_unit(tower_http::LatencyUnit::Micros)));

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