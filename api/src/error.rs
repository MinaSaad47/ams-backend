use axum::{http::StatusCode, response::IntoResponse, Json};
use logic::error::RepoError;
use sea_orm::sea_query::tests_cfg::json;

use crate::auth::AuthError;

pub enum ApiError {
    RepoError(RepoError),
    AuthError(AuthError),
    Unknown,
}

impl From<RepoError> for ApiError {
    fn from(inner: RepoError) -> Self {
        ApiError::RepoError(inner)
    }
}

impl From<AuthError> for ApiError {
    fn from(inner: AuthError) -> Self {
        ApiError::AuthError(inner)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, response) = match self {
            ApiError::RepoError(error) => match error {
                RepoError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    Json(json!({"status": false, "message": error.to_string()})),
                ),
                RepoError::Duplicate(_) => (
                    StatusCode::CONFLICT,
                    Json(json!({"status": false, "message": error.to_string()})),
                ),
                RepoError::Unknown => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"status": false, "message": "internal server error"})),
                ),
            },
            ApiError::Unknown => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": false, "message": "internal server error"})),
            ),
            ApiError::AuthError(error) => {
                let code = match error {
                    AuthError::WrongCredentials => StatusCode::UNAUTHORIZED,
                    AuthError::MissingCredentials => StatusCode::BAD_REQUEST,
                    AuthError::TokenCreation => StatusCode::INTERNAL_SERVER_ERROR,
                    AuthError::InvalidToken => StatusCode::BAD_REQUEST,
                    AuthError::UnauthorizedAccess => StatusCode::UNAUTHORIZED,
                };
                (
                    code,
                    Json(
                        json!({"status": false, "message": format!("auth error: {}", error.to_string())}),
                    ),
                )
            }
        };

        (status, response).into_response()
    }
}
