use std::net::SocketAddr;

use axum::{response::Html, routing::get, Router};

#[tokio::main]
async fn main() {
    let routes_hello = Router::new().route(
        "/hello",
        get(|| async { Html("Hellof <strong>World!!!</strong>") }),
    );
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("->> Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, routes_hello).await.unwrap();
}
