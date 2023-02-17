use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};

use logic::subjects::{CreateSubject, Subject};

use uuid::Uuid;

use crate::{error::ApiError, response::AppResponse, DynSubjectsRepo};

pub fn routes(subjects_repo: DynSubjectsRepo) -> Router {
    Router::new()
        .route("/subjects", get(get_all).post(create_one))
        .route("/subjects/:id", delete(delete_one))
        .with_state(subjects_repo)
}

async fn create_one(
    State(repo): State<DynSubjectsRepo>,
    Json(subject): Json<CreateSubject>,
) -> Result<AppResponse<Subject>, ApiError> {
    Ok(AppResponse::created(
        repo.create(subject).await?,
        "created a subject successfully",
    ))
}

async fn get_all(
    State(repo): State<DynSubjectsRepo>,
) -> Result<AppResponse<Vec<Subject>>, ApiError> {
    Ok(AppResponse::with_content(
        repo.get(None, None).await?,
        "retreived all subjects successfully",
    ))
}

pub async fn delete_one(
    Path(id): Path<Uuid>,
    State(repo): State<DynSubjectsRepo>,
) -> Result<AppResponse<()>, ApiError> {
    repo.delete_by_id(id).await?;
    Ok(AppResponse::no_content("deleted an subject successfully"))
}
