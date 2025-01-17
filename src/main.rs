use neurassembly::api;
use std::net::SocketAddr;
use tracing_subscriber;
use axum::serve;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create the API router
    let app = api::setup_router();

    // Bind to address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Starting server on {}", addr);

    // Start the server
    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app.into_make_service())
        .await
        .unwrap();
}