pub use self::error::{Error, Result};

use axum::{
    extract::{Path, Query},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use serde::Deserialize;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

mod error;
mod web;

#[tokio::main]
async fn main() {
    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        // middleware: layers are executed from bottom up
        .layer(middleware::map_response(main_response_mapper))
        .layer(CookieManagerLayer::new())
        // because of overlaps, we don't merge but fallback instead
        .fallback_service(routes_static());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, routes_all).await.unwrap();
}

async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    println!(); // empty line between requests
    res
}

/// allows to fallback to serving files: we have to provide the path to the file
/// in the url, for ex /src/main.rs
fn routes_static() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./")))
}

fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/:name", get(handler_hello2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

// /hello?name=Mike
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");
    let name = params.name.as_deref().unwrap_or("World");
    Html(format!("Hello <strong>{name}!!</strong>"))
}

// /hello2/Mike
// Note that Axum allows you to return Results as long as Ok and Err implements
// intoResponse
async fn handler_hello2(Path(name): Path<String>) -> Result<impl IntoResponse> {
    println!("->> {:<12} - handler_hello2 - {name:?}", "HANDLER");
    Ok(Html(format!("Hello <strong>{name}!!</strong>")))
}
