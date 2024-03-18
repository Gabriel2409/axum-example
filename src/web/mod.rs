mod error;
pub mod mw_auth;
pub mod mw_res_map;
pub mod routes_login;
pub mod routes_static;
pub mod rpc;

use tower_cookies::{Cookie, Cookies};

use crate::crypt::token::generate_web_token;

pub use self::error::ClientError;
pub use self::error::{Error, Result};

// endregion: --- Modules

pub const AUTH_TOKEN: &str = "auth-token";

fn set_token_cookie(cookies: &Cookies, user: &str, salt: &str) -> Result<()> {
    let token = generate_web_token(user, salt)?;
    let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
    // not accessible by javascript
    cookie.set_http_only(true);
    // the default is the uri part of the request (/api/login)
    // by setting the path at root, the cookie is available in the whole application
    cookie.set_path("/");
    cookies.add(cookie);
    Ok(())
}

fn remove_token_cookie(cookies: &Cookies) -> Result<()> {
    let mut cookie = Cookie::from(AUTH_TOKEN);
    cookie.set_path("/");
    cookies.remove(cookie);

    Ok(())
}
