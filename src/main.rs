use rewards_simulator::server::app;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let router = app();
    let addr: SocketAddr = "127.0.0.1:35025".parse().expect("valid addr");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, router).await.expect("server");
}
