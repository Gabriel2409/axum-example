use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use modql::field::HasFields;
use modql::filter::{FilterGroups, ListOptions};
use modql::SIden;
use sea_query::{Condition, Expr, Iden, IntoIden, PostgresQueryBuilder, Query, TableRef};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

const LIST_LIMIT_DEFAULT: i64 = 300;
const LIST_LIMIT_MAX: i64 = 1000;

#[derive(Iden)]
pub enum CommonIden {
    Id,
}

pub trait DbBmc {
    const TABLE: &'static str;

    fn table_ref() -> TableRef {
        TableRef::Table(SIden(Self::TABLE).into_iden())
    }
}

pub fn finalize_list_options(list_options: Option<ListOptions>) -> Result<ListOptions> {
    // When Some, validate limit
    match list_options {
        Some(mut list_options) => {
            // Validate the limit.
            match list_options.limit {
                Some(limit) => {
                    if limit > LIST_LIMIT_MAX {
                        return Err(Error::ListLimitOverMax {
                            max: LIST_LIMIT_MAX,
                            actual: limit,
                        });
                    }
                }
                None => {
                    list_options.limit = Some(LIST_LIMIT_DEFAULT);
                }
            }
            Ok(list_options)
        }
        None => Ok(ListOptions {
            limit: Some(LIST_LIMIT_DEFAULT),
            offset: None,
            order_bys: Some("id".into()),
        }),
    }
}

pub async fn create<MC, E>(_ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
    MC: DbBmc,
    E: HasFields,
{
    let db = mm.db();
    // Extract fields
    let fields = data.not_none_fields();

    let (columns, sea_values) = fields.for_sea_insert();

    // build query with seaquery - seaquery actually works across db drivers
    let mut query = Query::insert();
    query
        .into_table(MC::table_ref())
        .columns(columns)
        .values(sea_values)?
        .returning(Query::returning().columns([CommonIden::Id]));

    // Execute query with sqlx
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let (id,) = sqlx::query_as_with::<_, (i64,), _>(&sql, values)
        .fetch_one(db)
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
    // let entity: E = sqlb::select()
    //     .table(MC::TABLE)
    //     .columns(E::field_names()) // specific to sqlb, only select relevant fields
    //     .and_where("id", "=", id)
    //     // ----------
    //     .fetch_optional(db)
    //     .await?
    //     .ok_or(Error::EntityNotFound {
    //         entity: MC::TABLE,
    //         id,
    //     })?;
    // -------------------------------

    // snippet with seaquery + sqlx
    //

    let mut query = Query::select();
    query
        .from(MC::table_ref())
        .columns(E::field_column_refs())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })?;

    Ok(entity)
}

pub async fn list<MC, E, F>(
    _ctx: &Ctx,
    mm: &ModelManager,
    filter: Option<F>,
    list_options: Option<ListOptions>,
) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
    E: HasFields,
    F: Into<FilterGroups>,
{
    let db = mm.db();

    // Build query
    let mut query = Query::select();
    query.from(MC::table_ref()).columns(E::field_column_refs());

    // Filter conditions
    if let Some(filter) = filter {
        let filters: FilterGroups = filter.into();
        let cond: Condition = filters.try_into()?;
        query.cond_where(cond);
    }

    // list options
    let list_options = finalize_list_options(list_options)?;
    list_options.apply_to_sea_query(&mut query);

    // Exec query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    let entities = sqlx::query_as_with::<_, E, _>(&sql, values)
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
    // -- prep data
    let fields = data.not_none_fields();
    let fields = fields.for_sea_update();

    // -- build query
    let mut query = Query::update();
    query
        .table(MC::table_ref())
        .values(fields)
        .and_where(Expr::col(CommonIden::Id).eq(id));

    // -- exec query
    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(db)
        .await?
        .rows_affected();

    // -- check result
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
    let mut query = Query::delete();
    query
        .from_table(MC::table_ref())
        .and_where(Expr::col(CommonIden::Id).eq(id));

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let count = sqlx::query_with(&sql, values)
        .execute(db)
        .await?
        .rows_affected();

    if count == 0 {
        Err(Error::EntityNotFound {
            entity: MC::TABLE,
            id,
        })
    } else {
        Ok(())
    }
}
