use std::sync::Arc;

use crate::ctx::Ctx;
use crate::log::log_request;
use crate::web;
use crate::web::rpc::RpcInfo;
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, to_value};
use tracing::debug;
use uuid::Uuid;

pub async fn mw_reponse_map(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    debug!("{:<12} - mw_reponse_map", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    // -- extract rpc info
    let rpc_info = res.extensions().get::<Arc<RpcInfo>>().map(Arc::as_ref);
    // -- Get the eventual response error.
    // We access by type and we wrapped it in an Arc so we must use the Arc keyword
    let web_error = res.extensions().get::<Arc<web::Error>>().map(Arc::as_ref);
    let client_status_error = web_error.map(|se| se.client_status_and_error());

    // -- If client error, build the new reponse.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error = to_value(client_error).ok();
            let message = client_error.as_ref().and_then(|v| v.get("message"));
            let detail = client_error.as_ref().and_then(|v| v.get("detail"));

            let client_error_body = json!({
                "id": rpc_info.map(|rpc| rpc.id.clone()),
                "error": {
                    "message": message, // Variant name
                    "data": {
                        "req_uuid": uuid.to_string(),
                        "detail": detail
                    },
                }
            });

            debug!("CLIENT ERROR BODY:\n{client_error_body}");

            // Build the new response from the client_error_body
            (*status_code, Json(client_error_body)).into_response()
        });

    // -- Build and log the server log line.
    let client_error = client_status_error.unzip().1;
    // TODO: Need to hander if log_request fail (but should not fail request)
    let _ = log_request(
        uuid,
        req_method,
        uri,
        rpc_info,
        ctx,
        web_error,
        client_error,
    )
    .await;

    debug!("\n"); // FIXME: only used for local dev

    error_response.unwrap_or(res)
}
