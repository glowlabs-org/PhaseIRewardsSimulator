use rewards_simulator::server::app;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let router = app();

    let addr: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:35025".to_string())
        .parse()
        .unwrap_or_else(|_| "0.0.0.0:35025".parse().expect("fallback addr"));

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, router).await.expect("server");
}
