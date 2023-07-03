use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use uuid::Uuid;

use ams_logic::prelude::*;

use crate::{
    app::{self, DynSubjectsRepo},
    response::AppResponseDataExt,
};
use crate::{
    auth::{AuthError, Claims, User},
    error::ApiError,
    response::{AppResponse, AppResponseMsgExt},
};

pub(crate) fn routes() -> Router<app::State> {
    Router::new()
        .route("/subjects", get(get_all).post(create_one))
        .route(
            "/subjects/:id",
            get(get_one).patch(update_one).delete(delete_one),
        )
        .route("/subjects/:id/attendees", get(get_all_attendees))
        .route("/subjects/:id/subject_dates", post(add_one_subject_date))
        .route(
            "/subjects/:id/subject_dates/:id",
            delete(delete_one_subject_date),
        )
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

#[utoipa::path(
    post,
    path = "/subjects/{subject_id}/subject_dates",
    request_body = CreateSubjectDate,
    responses(
        (status = CREATED, body = SubjectDateResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn add_one_subject_date(
    State(repo): State<DynSubjectsRepo>,
    claims: Claims,
    Path(subject_id): Path<Uuid>,
    Json(subject_date): Json<CreateSubjectDate>,
) -> Result<AppResponse<'static, SubjectDate>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subject_date = repo.add_subject_date(subject_id, subject_date).await?;
    let response = subject_date.ok_response("added one subject date successfully");

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/subjects/{subject_id}/subject_dates/{subject_date_id}",
    responses(
        (status = OK)
    ),
    security(("api_jwt_token" = []))
)]
async fn delete_one_subject_date(
    State(repo): State<DynSubjectsRepo>,
    claims: Claims,
    Path((subject_id, subject_date_id)): Path<(Uuid, Uuid)>,
) -> Result<AppResponse<'static, ()>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.remove_subject_date(subject_id, subject_date_id)
        .await?;

    let response = "deleted one subject date successfully".response();

    Ok(response)
}
