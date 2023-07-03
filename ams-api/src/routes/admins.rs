use axum::{extract::State, routing::post, Json, Router};
use jsonwebtoken::{encode, Header};

use crate::{
    app::{self, DynAdminsRepo},
    auth::{AuthBody, AuthError, AuthPayload, Claims, User, KEYS},
    error::ApiError,
    response::AppResponse,
    response::AppResponseDataExt,
};

pub(crate) fn routes() -> Router<app::State> {
    Router::new().route("/admins/login", post(login))
}

#[utoipa::path(
    post,
    path = "/admins/login",
    request_body = AuthPayload,
    responses(
        (status = OK, body = AuthResponse)
    ),
)]
async fn login(
    State(repo): State<DynAdminsRepo>,
    payload: Option<Json<AuthPayload>>,
) -> Result<AppResponse<'static, AuthBody>, ApiError> {
    let Some(Json(payload)) = payload else {
        return Err(AuthError::MissingCredentials.into());
    };

    let admin = repo.get_by_email(payload.email).await?;

    if payload.password != admin.password {
        return Err(AuthError::WrongCredentials.into());
    }

    let claims = Claims {
        exp: usize::max_value(),
        user: User::Admin(admin.id),
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

    let response = AuthBody { token }.create_response("logged in as admin successfully");
    Ok(response)
}

