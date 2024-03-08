use crate::log::log_request;

pub use self::error::{Error, Result};

use axum::{
    extract::{Path, Query},
    http::{Method, Uri},
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Json, Router,
};
use ctx::Ctx;
use model::ModelController;
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod ctx;
mod error;
mod log;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize ModelController
    let mc = ModelController::new().await?;

    let routes_api = web::routes_tickets::routes(mc.clone())
        // apply middleware only for these routes
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(web::routes_login::routes())
        // merge but adds a prefix
        .nest("/api", routes_api)
        // middleware: layers are executed from bottom up
        .layer(middleware::map_response(main_response_mapper))
        // mw_ctx_resolver stores ctx in the request. This is an expensive operation so
        // that is why we want to only do it once
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        // because of overlaps, we don't merge but fallback instead
        .fallback_service(routes_static());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
    axum::serve(listener, routes_all).await.unwrap();

    Ok(())
}

/// Checks if there is an error and returns it as a JSON response.
/// If there is no error, just returns the Response.
async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");

    let uuid = Uuid::new_v4();

    // Get possible error
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                "type": client_error.as_ref(),
                "request_id": uuid.to_string(),
                }
            });
            println!("  ->> client_error_body: {client_error_body}");

            // Build the new response
            // Note that status_code implements Copy so a deref actually clone it
            // and it takes ownership
            (*status_code, Json(client_error_body)).into_response()
        });

    let client_error = client_status_error.unzip().1; // Option<ClientError>

    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    println!();
    error_response.unwrap_or(res)
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
