#![allow(unused)] // For early development.

// region:    --- Modules

// submodules first
mod config;
mod crypt;
mod ctx;
mod error;
mod log;
mod model;
mod utils;
mod web;

// #[cfg(test)] // Commented only for early development.
pub mod _dev_utils;

// then reexports
// NOTE: for reexports, you can start with crate:: so that it starts at the root
// module, self:: so that it starts at the current module or nothing (implicit, depends
// if you are on main.rs or not)
pub use self::error::{Error, Result};
use axum::response::Html;
use axum::routing::get;
pub use config::config;

// then imports
use crate::model::ModelManager;
use crate::web::mw_auth::{mw_ctx_require, mw_ctx_resolve};
use crate::web::mw_res_map::mw_reponse_map;
use crate::web::{routes_login, routes_static, rpc};
use axum::{middleware, Router};
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time() // omits timestamps from logs
        .with_target(false) // don't show file
        // filter logs based on the RUST_LOG env var
        // for ex with RUST_LOG=info,my_crate=debug
        // we would only use log info or above except for my_crate module
        // where we log debug or above
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // -- FIXME: FOR DEV ONLY
    _dev_utils::init_dev().await;

    // Initialize ModelManager.
    let mm = ModelManager::new().await?;

    // -- Define Routes
    let routes_rpc = rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_ctx_require));

    let routes_hello = Router::new()
        .route("/hello", get(|| async { Html("Hello world") }))
        .route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_login::routes(mm.clone()))
        .merge(routes_hello)
        .nest("/api", routes_rpc)
        .layer(middleware::map_response(mw_reponse_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    // region:    --- Start Server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
    axum::serve(listener, routes_all).await.unwrap();
    // endregion: --- Start Server

    Ok(())
}
