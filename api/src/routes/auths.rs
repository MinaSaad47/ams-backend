use axum::{extract::State, routing::post, Json, Router};
use jsonwebtoken::{encode, Header};

use crate::{
    auth::{AuthBody, AuthError, AuthPayload, Claims, KEYS},
    error::ApiError,
    response::AppResponse,
    DynUsersRepo,
};

pub fn routes(user_repo: DynUsersRepo) -> Router {
    Router::new()
        .route("/auth/login", post(login))
        .with_state(user_repo)
}

async fn login(
    State(repo): State<DynUsersRepo>,
    Json(payload): Json<AuthPayload>,
) -> Result<AppResponse<AuthBody>, ApiError> {
    let user = repo.get_by_email(payload.email).await?;

    if user.password != payload.password {
        Err(AuthError::WrongCredentials)?;
    }

    let claims = Claims {
        id: user.id,
        exp: usize::max_value(),
    };
    // Create the authorization token
    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    // Send the authorized token
    Ok(AppResponse::with_content(
        AuthBody { token },
        "created authorization token successfully",
    ))
}
