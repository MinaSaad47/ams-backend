use axum::{
    extract::{FromRef, Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use logic::prelude::*;

use crate::{
    auth::{AuthError, Claims, User},
    error::ApiError,
    response::{AppResponse, AppResponseDataExt, AppResponseMsgExt},
    DynAttendeesRepo, DynSubjectsRepo,
};

#[derive(FromRef, Clone)]
pub struct SubjectsState {
    pub subjects_repo: DynSubjectsRepo,
    pub attendees_repo: DynAttendeesRepo,
}

pub fn routes(subjects_state: SubjectsState) -> Router {
    Router::new()
        .route("/subjects", get(get_all).post(create_one))
        .route(
            "/subjects/:id",
            get(get_one).patch(update_one).delete(delete_one),
        )
        .route("/subjects/:id/attendees", get(get_all_attendees))
        .with_state(subjects_state)
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
) -> Result<AppResponse<'static, Vec<Subject>>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subjects = repo.get(SubjectsFilter::default()).await?;
    let response = subjects.ok_response("retreived all subjects successfully");

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
) -> Result<AppResponse<'static, Subject>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subjects = repo.create(subject).await?;
    let response = subjects.ok_response("retreived all subjects successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/subjects/{subject_id}",
    responses(
        (status = CREATED, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_one(
    State(repo): State<DynSubjectsRepo>,
    _: Claims,
    Path(subject_id): Path<Uuid>,
) -> Result<AppResponse<'static, Subject>, ApiError> {
    let subjects = repo.get_by_id(subject_id).await?;
    let response = subjects.ok_response("retreived one subject successfully");

    Ok(response)
}

#[utoipa::path(
    patch,
    path = "/subjects/{subject_id}",
    request_body = UpdateSubject,
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn update_one(
    State(repo): State<DynSubjectsRepo>,
    claims: Claims,
    Path(subject_id): Path<Uuid>,
    Json(subject): Json<UpdateSubject>,
) -> Result<AppResponse<'static, Subject>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subjects = repo.update(subject_id, subject).await?;
    let response = subjects.ok_response("updated one subject successfully");

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/subjects/{subject_id}",
    responses(
        (status = OK)
    ),
    security(("api_jwt_token" = []))
)]
async fn delete_one(
    State(repo): State<DynSubjectsRepo>,
    claims: Claims,
    Path(subject_id): Path<Uuid>,
) -> Result<AppResponse<'static, ()>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.delete_by_id(subject_id).await?;
    let response = "deleted one subject successfully".response();

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/subjects/{subject_id}/attendees",
    responses(
        (status = OK, body = AttendeesListResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_all_attendees(
    State(repo): State<DynSubjectsRepo>,
    Path(subject_id): Path<Uuid>,
    claims: Claims,
) -> Result<AppResponse<'static, Vec<Attendee>>, ApiError> {
    if let User::Attendee(_) = claims.user {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subjects = repo.get_all_attendees(subject_id).await?;

    let response = subjects.ok_response("retreived all attendees successfully");

    Ok(response)
}
