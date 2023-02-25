use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    RequestPartsExt,
};
use dotenvy_macro::dotenv;
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Validation};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::{ToResponse, ToSchema};
use uuid::Uuid;

use crate::error::ApiError;

pub static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = dotenv!("JWT_SECRET");
    Keys::new(secret.as_bytes())
});

#[derive(Serialize, Deserialize, Debug)]
pub enum User {
    Admin(Uuid),
    Instructor(Uuid),
    Attendee(Uuid),
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user: User,
    pub exp: usize,
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

#[derive(Serialize, Deserialize, ToResponse, ToSchema)]
pub struct AuthBody {
    pub token: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AuthPayload {
    #[schema(example = "mina@saad.com")]
    pub email: String,
    #[schema(example = "474747")]
    pub password: String,
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum AuthError {
    #[error("wrong credentials")]
    WrongCredentials,
    #[error("unauthorization access")]
    UnauthorizedAccess,
    #[error("missing credentials")]
    MissingCredentials,
    #[error("token creation error")]
    TokenCreation,
    #[error("invalid token")]
    InvalidToken,
}

pub struct Keys {
    pub encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
