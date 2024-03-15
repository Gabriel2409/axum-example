use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use sqlb::HasFields;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

pub trait DbBmc {
    const TABLE: &'static str;
}

pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();
    let fields = data.not_none_fields();
    let (id,) = sqlb::insert()
        .table(MC::TABLE)
        .data(fields)
        .returning(&["id"])
        .fetch_one::<_, (i64,)>(db)
        .await?;
    Ok(id)
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    MC: DbBmc,
    // for FromRow we need a lifetime
    // Unpin means it can be moved freely in memory, important for async
    // Send means that it can be sent across threads
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields, // trait for sqlb
{
    let db = mm.db();

    // -----------
    // // snippet with sqlx
    // let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE);
    // let entity: E = sqlx::query_as(&sql)
    //     .bind(id)
    // ----------
    // snippet with sqlb
    // by default sqlb will do a select *
    let entity: E = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names()) // specific to sqlb, only select relevant fields
        .and_where("id", "=", id)
        // ----------
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;
    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
{
    let db = mm.db();

    let entities: Vec<E> = sqlb::select()
        .table(MC::TABLE)
        .columns(E::field_names()) // specific to sqlb, only select relevant fields
        // ----------
        .order_by("id")
        .fetch_all(db)
        .await?;
    Ok(entities)
}

pub async fn update<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64, data: E) -> Result<()>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();
    let fields = data.not_none_fields();
    let count = sqlb::update()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .data(fields)
        .returning(&["id"])
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
    MC: DbBmc,
{
    let db = mm.db();
    let count = sqlb::delete()
        .table(MC::TABLE)
        .and_where("id", "=", id)
        .exec(db)
        .await?;

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}