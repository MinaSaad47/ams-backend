use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use logic::prelude::*;

use crate::{
    auth::{AuthError, Claims, User},
    error::ApiError,
    response::AppResponse,
    DynSubjectsRepo,
};

pub fn routes(subjects_repo: DynSubjectsRepo) -> Router {
    Router::new()
        .route("/subjects", get(get_all).post(create_one))
        .route(
            "/subjects/:id",
            get(get_one).patch(update_one).delete(delete_one),
        )
        .with_state(subjects_repo)
}

#[utoipa::path(
    get,
    path = "/subjects",
    responses(
        (status = OK, body = SubjectsListResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_all(
    State(repo): State<DynSubjectsRepo>,
    claims: Claims,
) -> Result<AppResponse<Vec<Subject>>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subjects = repo.get(SubjectsFilter::default()).await?;
    let response = AppResponse::with_content(subjects, "retreived all subjects successfully");

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/subjects",
    request_body = CreateSubject,
    responses(
        (status = CREATED, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn create_one(
    State(repo): State<DynSubjectsRepo>,
    claims: Claims,
    Json(subject): Json<CreateSubject>,
) -> Result<AppResponse<Subject>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subjects = repo.create(subject).await?;
    let response = AppResponse::with_content(subjects, "retreived all subjects successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/subjects/{id}",
    params(
        ("id" = Uuid, Path, description = "subject id"),
    ),
    responses(
        (status = CREATED, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_one(
    State(repo): State<DynSubjectsRepo>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Result<AppResponse<Subject>, ApiError> {
    let subjects = repo.get_by_id(id).await?;
    let response = AppResponse::with_content(subjects, "retreived one subject successfully");

    Ok(response)
}

#[utoipa::path(
    patch,
    path = "/subjects/{id}",
    params(
        ("id" = Uuid, Path, description = "subject id"),
    ),
    request_body = UpdateSubject,
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn update_one(
    State(repo): State<DynSubjectsRepo>,
    claims: Claims,
    Path(id): Path<Uuid>,
    Json(subject): Json<UpdateSubject>,
) -> Result<AppResponse<Subject>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subjects = repo.update(id, subject).await?;
    let response = AppResponse::with_content(subjects, "updated one subject successfully");

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/subjects/{id}",
    params(
        ("id" = Uuid, Path, description = "subject id"),
    ),
    responses(
        (status = OK)
    ),
    security(("api_jwt_token" = []))
)]
async fn delete_one(
    State(repo): State<DynSubjectsRepo>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<AppResponse<()>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.delete_by_id(id).await?;
    let response = AppResponse::no_content("deleted one subject successfully");

    Ok(response)
}
