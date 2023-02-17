use std::ops::Deref;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State, TypedHeader},
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    RequestPartsExt,
};
use dotenvy_macro::dotenv;
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Validation};
use logic::users::{User, UserRole};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{error::ApiError, DynUsersRepo};

pub static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = dotenv!("JWT_SECRET");
    Keys::new(secret.as_bytes())
});

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: Uuid,
    pub exp: usize,
}

pub struct Instructor {
    inner: User,
}

impl Deref for Instructor {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct Attendee {
    inner: User,
}

impl Deref for Attendee {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Attendee
where
    S: Send + Sync,
    DynUsersRepo: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = extract_user_from_token(parts, state).await?;

        if matches!(user.role, UserRole::Instructor) {
            Err(AuthError::UnauthorizedAccess)?;
        }

        Ok(Attendee { inner: user })
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Instructor
where
    S: Send + Sync,
    DynUsersRepo: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = extract_user_from_token(parts, state).await?;

        if matches!(user.role, UserRole::Attendee) {
            Err(AuthError::UnauthorizedAccess)?;
        }

        Ok(Instructor { inner: user })
    }
}

async fn extract_user_from_token<S>(parts: &mut Parts, state: &S) -> Result<User, ApiError>
where
    S: Send + Sync,
    DynUsersRepo: FromRef<S>,
{
    // Extract the token from the authorization header
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| AuthError::InvalidToken)?;
    // Decode the user data
    let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
        .map_err(|_| AuthError::InvalidToken)?;

    let State(user_repo): State<DynUsersRepo> = parts
        .extract_with_state(state)
        .await
        .map_err(|_| ApiError::Unknown)?;

    Ok(user_repo.get_by_id(token_data.claims.id).await?)
}

#[derive(Serialize)]
pub struct AuthBody {
    pub token: String,
}

#[derive(Deserialize)]
pub struct AuthPayload {
    pub email: String,
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
