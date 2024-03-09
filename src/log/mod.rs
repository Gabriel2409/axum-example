use std::time::{SystemTime, UNIX_EPOCH};

use crate::ctx::Ctx;
use crate::error::ClientError;
use crate::{Error, Result};
use axum::http::{Method, Uri};
use serde::Serialize;
use serde_json::{json, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

pub async fn log_request(
    uuid: Uuid,
    req_method: Method,
    uri: Uri,
    ctx: Option<Ctx>,
    web_error: Option<&Error>,
    client_error: Option<ClientError>,
) -> Result<()> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let error_type = web_error.map(|se| se.as_ref().to_string());
    let error_data = serde_json::to_value(web_error)
        .ok()
        .and_then(|mut v| v.get_mut("data").map(|v| v.take()));

    let log_line = RequestLogLine {
        uuid: uuid.to_string(),
        timestamp: timestamp.to_string(),
        user_id: ctx.map(|c| c.user_id()),
        http_path: uri.to_string(),
        http_method: req_method.to_string(),
        client_error_type: client_error.map(|ce| ce.as_ref().to_string()),
        error_type,
        error_data,
    };
    println!("->> REQUEST LOG LINE:\n{}", json!(log_line));

    // TODO: - send to cloud watch
    Ok(())
}

#[skip_serializing_none]
#[derive(Serialize)]
struct RequestLogLine {
    uuid: String,      // uuid string formatted
    timestamp: String, // iso8601

    // -- user and context attribute
    user_id: Option<u64>,

    // -- http request attributes
    http_path: String,
    http_method: String,

    // -- Errors attributes
    client_error_type: Option<String>,
    error_type: Option<String>,
    error_data: Option<Value>,
}
