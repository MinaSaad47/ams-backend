use axum::{
    extract::{FromRef, State},
    routing::post,
    Json, Router,
};
use jsonwebtoken::{encode, Header};

use crate::{
    auth::{AuthBody, AuthError, AuthPayload, Claims, User, KEYS},
    error::ApiError,
    response::AppResponse,
    DynAdminsRepo,
};

#[derive(Clone, FromRef)]
pub struct AdminsState {
    pub admins_repo: DynAdminsRepo,
}

pub fn routes(admins_state: AdminsState) -> Router {
    Router::new()
        .route("/admins/login", post(login_one_admin))
        .with_state(admins_state)
}

async fn login_one_admin(
    State(repo): State<DynAdminsRepo>,
    Json(payload): Json<AuthPayload>,
) -> Result<AppResponse<AuthBody>, ApiError> {
    let admin = repo.get_by_email(payload.email).await?;

    if payload.password != admin.password {
        return Err(AuthError::WrongCredentials.into());
    }

    let claims = Claims {
        exp: usize::max_value(),
        user: User::Admin(admin.id),
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

    let response = AppResponse::with_content(AuthBody { token }, "logged in as admin successfully");

    Ok(response)
}
