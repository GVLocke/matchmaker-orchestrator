mod requests;
use axum::{routing::get, Router};
use requests::handle_upload;

#[tokio::main]
async fn main() {
    // Create the axum router
    let router01 = Router::new()
        .route("/scrape", get(handle_upload));

    // Define the IP and port listener (TCP)
    let address = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    // Call axum serve to launch the web server
    axum::serve(listener, router01).await.unwrap();
}