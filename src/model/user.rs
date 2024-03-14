use crate::ctx::Ctx;

use crate::model::base::{self, DbBmc};
use crate::model::ModelManager;
use crate::model::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
}

// for app api, arguments of UserBmc::create
#[derive(Deserialize)]
pub struct UserForCreate {
    pub username: String,
    pub pwd_clear: String,
}

// for user module implementation, inside the UserBmc::create func
#[derive(Deserialize)]
struct UserForInsert {
    username: String,
}

// struct to validate the login
#[derive(Debug, Clone, Fields, FromRow)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,
    pub pwd: Option<String>, // encrypted
    pub pwd_salt: Uuid,
    pub token_salt: Uuid,
}

#[derive(Debug, Clone, Fields, FromRow)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,
    pub token_salt: Uuid,
}

/// Marker trait
pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

pub struct UserBmc;
impl DbBmc for UserBmc {
    const TABLE: &'static str = "user";
}

impl UserBmc {
    // ensures only the User, UserForLogin and UserForAuth can be used
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: UserBy,
    {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn first_by_username<E>(
        _ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserBy,
    {
        let db = mm.db();

        let user = sqlb::select()
            .table(Self::TABLE)
            .and_where("username", "=", username)
            .fetch_optional(db)
            .await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::_dev_utils;
    use anyhow::{Context, Result};
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_first_ok_demo1() -> Result<()> {
        // -- Setup & Fixtures
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_username = "demo1";

        // -- Exec
        let user: User = UserBmc::first_by_username(&ctx, &mm, fx_username)
            .await?
            .context("Should have user 'demo1'")?;

        // -- Check
        assert_eq!(user.username, fx_username);

        Ok(())
    }
}
