use rewards_simulator::server::app;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let router = app();

    // Prefer explicit BIND_ADDR, otherwise honor PORT (Railway/Heroku style),
    // and finally fall back to 0.0.0.0:35025 for local dev.
    let addr: SocketAddr = if let Ok(explicit) = std::env::var("BIND_ADDR") {
        explicit
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:35025".parse().expect("fallback addr"))
    } else if let Ok(port) = std::env::var("PORT") {
        let bind = format!("0.0.0.0:{port}");
        bind.parse()
            .unwrap_or_else(|_| "0.0.0.0:35025".parse().expect("fallback addr"))
    } else {
        "0.0.0.0:35025"
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:35025".parse().expect("fallback addr"))
    };

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
    axum::serve(listener, router).await.expect("server");
}
