use std::borrow::Cow;

use axum::{http::StatusCode, response::IntoResponse, Json};
use nn_model::EmbeddingError;
use sea_orm::sea_query::tests_cfg::json;
use thiserror::Error;
use tokio::io;

use logic::prelude::*;

use crate::auth::AuthError;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("database error happened: {0:#?}")]
    RepoError(#[from] RepoError),
    #[error("{0:#?}")]
    AuthError(#[from] AuthError),
    #[error("embedding error happened: {0:#?}")]
    EmbeddingError(#[from] EmbeddingError),
    #[error("io error happened: {0:#?}")]
    IOError(#[from] io::Error),
    #[error("setup error happened: {0:#?}")]
    SetupError(Cow<'static, str>),
    #[error("Unknown error happened")]
    Unknown,
}

impl IntoResponse for ApiError {
    #[tracing::instrument(ret(Debug), level = "error")]
    fn into_response(self) -> axum::response::Response {
        let (status, response) = match self {
            ApiError::RepoError(error) => match error {
                RepoError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    Json(
                        json!({"code": StatusCode::NOT_FOUND.as_u16(), "status": false, "message": error.to_string()}),
                    ),
                ),
                RepoError::Duplicate(_, _) => (
                    StatusCode::CONFLICT,
                    Json(
                        json!({"code": StatusCode::CONFLICT.as_u16(), "status": false, "message": error.to_string()}),
                    ),
                ),
                RepoError::Unknown => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(
                        json!({"code": StatusCode::INTERNAL_SERVER_ERROR.as_u16(), "status": false, "message": "internal server error"}),
                    ),
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
                    Json(json!({"status": false, "message": format!("auth error: {}", error)})),
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
