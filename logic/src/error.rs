use sea_orm::{DbErr, RuntimeErr};
use sqlx::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("no entity found: {0}")]
    NotFound(String),
    #[error("entity already exists: {0}")]
    Duplicate(String),
    #[error("unknown repository error")]
    Unknown,
}

impl From<DbErr> for RepoError {
    fn from(value: DbErr) -> Self {
        match value {
            DbErr::Exec(RuntimeErr::SqlxError(Error::Database(error)))
                if error.code().unwrap() == "23505" =>
            {
                Self::Duplicate(error.message().to_owned())
            }
            DbErr::Exec(RuntimeErr::SqlxError(error)) if matches!(error, Error::RowNotFound) => {
                Self::NotFound(error.as_database_error().unwrap().message().to_owned())
            }
            _ => Self::Unknown,
        }
    }
}
