use axum::{http::StatusCode, response::IntoResponse, Json};
use logic::prelude::RepoError;
use nn_model::EmbeddingError;
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use tokio::io;

use crate::auth::AuthError;

#[allow(dead_code)]
#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ApiError {
    #[error("record not found")]
    NotFound,
    #[error("record already exists")]
    Duplicate,
    #[error("unauthorized access")]
    Unauthorized { message: String },
    #[error("face could not be recogized")]
    FaceRecogition,
    #[error("bad request")]
    BadRequest,
    #[error("internal server error")]
    Internal,
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        #[allow(unused)]
        match self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::Duplicate => StatusCode::CONFLICT,
            ApiError::Unauthorized { message } => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<EmbeddingError> for ApiError {
    #[tracing::instrument(level = "error")]
    fn from(error: EmbeddingError) -> Self {
        Self::FaceRecogition
    }
}

impl From<io::Error> for ApiError {
    #[tracing::instrument(level = "error")]
    fn from(error: io::Error) -> Self {
        Self::Internal
    }
}

impl From<RepoError> for ApiError {
    #[tracing::instrument(level = "error")]
    fn from(error: RepoError) -> Self {
        #[allow(unused)]
        match error {
            RepoError::SubjectNotFound { id }
            | RepoError::AttendeeNotFound { id }
            | RepoError::InstructorNotFound { id }
            | RepoError::AttendanceNotFound { id }
            | RepoError::AdminNotFound { id } => Self::NotFound,
            RepoError::NotFound(message) => Self::NotFound,
            RepoError::DuplicateSubject
            | RepoError::DuplicateAttendee
            | RepoError::DuplicateInstructor
            | RepoError::DuplicateAttendance
            | RepoError::DuplicateAdmin
            | RepoError::Duplicate(_, _) => Self::Duplicate,
            RepoError::Unknown => Self::Internal,
        }
    }
}

impl From<AuthError> for ApiError {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::WrongCredentials
            | AuthError::UnauthorizedAccess
            | AuthError::MissingCredentials => Self::Unauthorized {
                message: value.to_string(),
            },
            AuthError::InvalidToken => Self::BadRequest,
            _ => Self::Internal,
        }
    }
}

impl IntoResponse for ApiError {
    #[tracing::instrument(ret(Debug), level = "error")]
    fn into_response(self) -> axum::response::Response {
        (
            self.status_code(),
            Json(json!({"status": "failure", "error": self})),
        )
            .into_response()
    }
}
