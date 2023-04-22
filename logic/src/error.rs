use sea_orm::{DbErr, RuntimeErr};
use serde::Serialize;
use sqlx::{postgres::PgDatabaseError, Error};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug, Serialize)]
pub enum RepoError {
    #[error("subject `{id}` not found")]
    SubjectNotFound { id: String },
    #[error("attendee `{id}` not found")]
    AttendeeNotFound { id: String },
    #[error("instructor `{id}` not found")]
    InstructorNotFound { id: String },
    #[error("attendance `{id}` not found")]
    AttendanceNotFound { id: String },
    #[error("admin `{id}` not found")]
    AdminNotFound { id: String },

    #[error("subject already exists")]
    DuplicateSubject,
    #[error("attendee already exists")]
    DuplicateAttendee,
    #[error("instructor already exists")]
    DuplicateInstructor,
    #[error("attendance already exists")]
    DuplicateAttendance { attendee_id: Uuid, subject_id: Uuid },
    #[error("admin already exists")]
    DuplicateAdmin,

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

pub(crate) trait MapDuplicateExt {
    type Output;
    fn map_duplicate(self, duplicate_error: RepoError) -> Result<Self::Output, RepoError>
    where
        Self: Sized;
}

impl<S> MapDuplicateExt for Result<S, DbErr> {
    type Output = S;
    fn map_duplicate(self, duplicate_error: RepoError) -> Result<Self::Output, RepoError>
    where
        Self: Sized,
    {
        self.map_err(|error| {
            let error = error.into();
            if let RepoError::Duplicate(_, _) = error {
                duplicate_error
            } else {
                error
            }
        })
    }
}
