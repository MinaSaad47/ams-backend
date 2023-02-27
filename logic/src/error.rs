use sea_orm::{DbErr, RuntimeErr};
use sqlx::{postgres::PgDatabaseError, Error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("duplicate in {0} `{1}`")]
    Duplicate(String, String),
    #[error("no record found in {0}")]
    NotFound(String),
    #[error("unknown repository error")]
    Unknown,
}

impl From<DbErr> for RepoError {
    fn from(value: DbErr) -> Self {
        tracing::debug!("{value:#?}");
        match value {
            DbErr::Query(RuntimeErr::SqlxError(rt_err))
            | DbErr::Exec(RuntimeErr::SqlxError(rt_err)) => {
                if let Some(pg_err) = rt_err
                    .as_database_error()
                    .and_then(|db_err| db_err.try_downcast_ref::<PgDatabaseError>())
                {
                    if let ("23505", Some(table), Some(detail)) =
                        (pg_err.code(), pg_err.table(), pg_err.detail())
                    {
                        return Self::Duplicate(table.to_owned(), detail.to_owned());
                    } else if matches!(Error::RowNotFound, _rt_err) {
                        if let Some(table) = pg_err.table() {
                            return Self::NotFound(table.to_owned());
                        }
                    }
                }
            }
            _ => {}
        };
        return Self::Unknown;
    }
}
