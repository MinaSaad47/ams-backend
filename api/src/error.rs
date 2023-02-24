use axum::{http::StatusCode, response::IntoResponse, Json};
use logic::error::RepoError;
use nn_model::EmbeddingError;
use sea_orm::sea_query::tests_cfg::json;
use thiserror::Error;
use tokio::io;

use crate::auth::AuthError;

#[allow(dead_code)]
#[derive(Error, Debug)]
#[error("api error")]
pub enum ApiError {
    RepoError(#[from] RepoError),
    AuthError(#[from] AuthError),
    EmbeddingError(#[from] EmbeddingError),
    IOError(#[from] io::Error),
    Unknown,
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
            ApiError::EmbeddingError(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": false, "message": error.to_string()})),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": false, "message": "internal server error"})),
            ),
        };

        (status, response).into_response()
    }
}
