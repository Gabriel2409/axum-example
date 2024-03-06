use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};

use crate::{web, Error, Result};

pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login))
}

async fn api_login(cookies: Cookies, payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:12} - api_login", "HANDLER");
    if payload.username != "admin" || payload.pwd != "password" {
        return Err(Error::LoginFail);
    }

    //FIXME: Implement real auth-token generation signature
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign"));

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
