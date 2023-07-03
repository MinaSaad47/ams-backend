use ams_facerec::FaceRecognitionError;
use ams_logic::prelude::RepoError;
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use tokio::io;
use uuid::Uuid;

use crate::auth::AuthError;

#[allow(dead_code)]
#[derive(Error, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ApiError {
    #[error("record not found")]
    NotFound { message: String },
    #[error("record already exists")]
    Duplicate { message: String },
    #[error("unauthorized access")]
    Unauthorized { message: String },
    #[error("face could not be recogized")]
    FaceRecogition,
    #[error("bad request")]
    BadRequest,
    #[error("internal server error")]
    Internal,
    #[error("attendance already taken")]
    #[serde(rename_all = "camelCase")]
    DuplicateAttendance { subject_id: Uuid, attendee_id: Uuid },
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        #[allow(unused)]
        match self {
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::Duplicate { .. } => StatusCode::CONFLICT,
            ApiError::Unauthorized { message } => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<FaceRecognitionError> for ApiError {
    #[tracing::instrument(level = "error")]
    fn from(error: FaceRecognitionError) -> Self {
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
        match &error {
            RepoError::SubjectNotFound { id }
            | RepoError::AttendeeNotFound { id }
            | RepoError::InstructorNotFound { id }
            | RepoError::AttendanceNotFound { id }
            | RepoError::AdminNotFound { id } => Self::NotFound {
                message: error.to_string(),
            },
            RepoError::NotFound(message) => Self::NotFound {
                message: message.clone(),
            },
            RepoError::DuplicateSubject
            | RepoError::DuplicateAttendee
            | RepoError::DuplicateInstructor
            | RepoError::DuplicateAdmin
            | RepoError::Duplicate(_, _) => Self::Duplicate {
                message: error.to_string(),
            },
            RepoError::DuplicateAttendance {
                attendee_id,
                subject_id,
            } => Self::DuplicateAttendance {
                subject_id: *subject_id,
                attendee_id: *attendee_id,
            },
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
