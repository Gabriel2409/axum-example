use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{Error, Result};

pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login))
}

async fn api_login(payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:12} - api_login", "HANDLER");
    if payload.username != "admin" || payload.pwd != "password" {
        return Err(Error::LoginFail);
    }

    let body = Json(json!({
        "result":{
        "success": true}

    }));
    Ok(body)
}

#[derive(Debug, Default, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}
