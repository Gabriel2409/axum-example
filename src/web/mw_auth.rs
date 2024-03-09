use async_trait::async_trait;

use axum::{
    body::Body,
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
    RequestPartsExt,
};
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};

use crate::{ctx::Ctx, model::ModelController, web::AUTH_TOKEN, Error, Result};

/// This function should take extractors as first arguments and next as a final arg.
/// Extractors must implement FromRequestParts. Here Request is provided by axum and
/// Ctx is custom made
/// There are 3 ways to inject an extractor: directly, as a result or an option:
/// - If i pass Ctx directly, then the method is called before passing the ctx to the
/// middleware
/// - if I pass it as a result, the middleware is called first and I can call `ctx?;`
/// so that it throws an error if it fails
/// - if I pass it as an option, it will be None on failure but it won't stop the
/// execution of the middleware
/// Check around 55:00
pub async fn mw_require_auth(ctx: Result<Ctx>, req: Request<Body>, next: Next) -> Result<Response> {
    println!("->> {:12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

/// Because ctx can be called several times, (for ex in the mw_require_auth and
/// in specific routes, we want its method from_request_parts to be as fast as possible
/// To do so, we implements the expensive operations only once, in a middleware, that
/// will be called on every request
pub async fn mw_ctx_resolver(
    _mc: State<ModelController>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    println!("->> {:12} - mw_ctx_resolver", "MIDDLEWARE");
    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());

    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token)
    {
        Ok((user_id, _exp, _sign)) => {
            // TODO: Token components validation - this is expensive
            Ok(Ctx::new(user_id))
        }
        Err(e) => Err(e),
    };

    // Now that we have the result, we don't want to fail in case of error
    // because some routes do not require auth. However, we want to clean up the
    // cookie if something went wrong
    if result_ctx.is_err() && !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN));
    }

    // Trick is to store the ctx_result in the request extension so that we can access
    // it later.
    // We use a datastore by type so if we give another Result<Ctx>, it will overwrite it
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

// impl of the trait for ctx so that we can use it as an extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");
        // No need to do the cookie parsing anymore, it is done my mw_ctx_resolver
        // So instead we get the ctx from the request
        parts
            .extensions
            // we get what was stored in the request as an option
            .get::<Result<Ctx>>() // <Option<&Result<Ctx>>
            // we transform the option in a result and use ? to extract
            .ok_or(Error::AuthFailCtxNotInRequestExt)? // &Result<Ctx>
            .clone() // Result<Ctx>
    }
}

fn parse_token(token: String) -> Result<(i64, String, String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(
        r#"^user-(\d+)\.(.+)\.(.+)"#, // literal regex
        &token
    )
    .ok_or(Error::AuthFailTokenWrongFormat)?;

    let user_id: i64 = user_id
        .parse()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;

    Ok((user_id, exp.to_string(), sign.to_string()))
}
