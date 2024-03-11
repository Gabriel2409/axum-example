use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::Result;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
}

#[derive(Deserialize)]
pub struct TaskForCreate {
    pub title: String,
}

#[derive(Deserialize)]
pub struct TaskForUpdate {
    pub title: Option<String>,
}

pub struct TaskBmc;

impl TaskBmc {
    pub async fn create(_ctx: &Ctx, mm: &ModelManager, task_c: TaskForUpdate) -> Result<i64> {
        let db = mm.db();
        // always do prepare statements to avoid sql injection
        let (id,) =
            sqlx::query_as::<_, (i64,)>("INSERT INTO task (title) values ($1) RETURNING id")
                .bind(task_c.title)
                .fetch_one(db)
                .await?;
        Ok(id)
    }
}
