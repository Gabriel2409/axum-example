use derive_more::From;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

use crate::{crypt, model::store};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
    EntityNotFound {
        entity: &'static str,
        id: i64,
    },
    // -- Modules
    // instead of manually implmenting From, we can use derive_more::From trait
    #[from]
    Crypt(crypt::Error),
    #[from]
    Store(store::Error),

    // -- Externals

    // because sqlx::Error does not implement Serialize,
    // but implements Display and FromStr, we use serde
    #[from]
    Sqlx(#[serde_as(as = "DisplayFromStr")] sqlx::Error),

    #[from]
    SeaQuery(#[serde_as(as = "DisplayFromStr")] sea_query::error::Error),
}

// Below lines not needed anymore now that we use From trait from derive_more
// impl From<crypt::Error> for Error {
//     fn from(val: crypt::Error) -> Self {
//         Self::Crypt(val)
//     }
// }
// impl From<store::Error> for Error {
//     fn from(val: store::Error) -> Self {
//         Self::Store(val)
//     }
// }
//
// impl From<sqlx::Error> for Error {
//     fn from(val: sqlx::Error) -> Self {
//         Self::Sqlx(val)
//     }
// }

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate
