//! Model Layer
//!
//! Design:
//!
//! - The Model layer normalizes the application's data type
//!   structures and access.
//! - All application code data access must go through the Model layer.
//! - The `ModelManager` holds the internal states/resources
//!   needed by ModelControllers to access data.
//!   (e.g., db_pool, S3 client, redis client).
//! - Model Controllers (e.g., `TaskBmc`, `ProjectBmc`) implement
//!   CRUD and other data access methods on a given "entity"
//!   (e.g., `Task`, `Project`).
//!   (`Bmc` is short for Backend Model Controller).
//! - In frameworks like Axum, Tauri, `ModelManager` are typically used as App State.
//! - ModelManager are designed to be passed as an argument
//!   to all Model Controllers functions.

// region:    --- Modules

mod base;
mod error;
mod store;
pub mod task;
pub mod user;

pub use self::error::{Error, Result};
use self::store::{new_db_pool, Db};

// endregion: --- Modules

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        // because our Error implements from for store errors, Error from store
        // is automatically mapped to Error from model.
        // ? is equivalent to map_err(|e| Error::from(e))?;
        let db = new_db_pool().await?;

        // FIXME: - TBC
        Ok(ModelManager { db })
    }

    /// only accessible within the model module
    /// Idea is that the sqlx db pool ref is only for the model layer
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }
}
