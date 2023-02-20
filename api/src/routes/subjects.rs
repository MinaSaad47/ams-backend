use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};

use logic::subjects::{CreateSubject, Subject, SubjectsFilter, UpdateSubject};
use uuid::Uuid;

use crate::{
    auth::{AuthError, Claims, User},
    error::ApiError,
    response::AppResponse,
    DynSubjectsRepo,
};

pub fn routes(subjects_repo: DynSubjectsRepo) -> Router {
    Router::new()
        .route("/subjects", get(get_all_subjects).post(create_one_subject))
        .route(
            "/subjects/:id",
            get(get_one_subject)
                .patch(update_one_subject)
                .delete(delete_one_subject),
        )
        .with_state(subjects_repo)
}

async fn get_all_subjects(
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

async fn create_one_subject(
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

async fn get_one_subject(
    State(repo): State<DynSubjectsRepo>,
    _: Claims,
    Path(id): Path<Uuid>,
) -> Result<AppResponse<Subject>, ApiError> {
    let subjects = repo.get_by_id(id).await?;
    let response = AppResponse::with_content(subjects, "retreived one subject successfully");

    Ok(response)
}

async fn update_one_subject(
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

async fn delete_one_subject(
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
