use crate::crypt::token::{validate_web_token, Token};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForAuth};
use crate::model::ModelManager;
use crate::web::AUTH_TOKEN;
use crate::web::{Error, Result};
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

use super::set_token_cookie;

#[allow(dead_code)] // For now, until we have the rpc.

/// Checks that there is no error in the ctx. If there is, returs early.
/// Under the hood, checks the from_request_parts method
/// Note that because it is passed as a result, the debut print is printed even if there
/// is an error.
/// If we passed it directly (and removed ctx?) it would not execute the function in
/// case of error.
/// Note that we can also pass it as an option but in this case, it would not error out
/// if it is None.
pub async fn mw_ctx_require(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolve(
    mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

    // should NOT fail if there is an error. It is the responsibility of the ctx auth
    // or other things downstream, so no ?
    let ctx_ext_result = _ctx_resolve(mm, &cookies).await;

    // Remove the cookie if something went wrong because we don't want to keep validating
    // a cookie that already failed once
    if ctx_ext_result.is_err() && !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN))
    }

    // Store the ctx_result in the request extension.
    req.extensions_mut().insert(ctx_ext_result);

    Ok(next.run(req).await)
}

async fn _ctx_resolve(mm: State<ModelManager>, cookies: &Cookies) -> CtxExtResult {
    // -- Get token string

    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;

    // -- Parse token
    // we can use parse because Token has FromStr
    // Note that we map the err because we don't want crypt to be a sub error of ctx
    let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;
    // let token = token.parse::<Token>().unwrap(); // other way to parse

    // -- Get UserForAuth
    let user: UserForAuth = UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
        .await
        .map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
        .ok_or(CtxExtError::UserNotFound)?;

    // -- Validate token
    validate_web_token(&token, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::FailValidate)?;

    // -- Update token
    set_token_cookie(cookies, &user.username, &user.token_salt.to_string())
        .map_err(|_| CtxExtError::CanNotSetTokenCookie)?;

    // -- Create CtxExtResult, it is independent from the web layer now that the
    // validation is done
    Ctx::new(user.id).map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region:    --- Ctx Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        debug!("{:<12} - Ctx", "EXTRACTOR");

        parts
            .extensions
            // checks in the dict by type, was set previously during mw_ctx_resolve
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            // wraps the error in a web error
            .map_err(Error::CtxExt)
    }
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<Ctx, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    TokenNotInCookie,
    TokenWrongFormat,
    UserNotFound,             // we don't capture the name
    ModelAccessError(String), // we don't want the full model error over there
    FailValidate,
    CanNotSetTokenCookie,
    CtxNotInRequestExt,
    CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error
