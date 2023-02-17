use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};

use logic::admins::{Admin, CreateAdmin};

use uuid::Uuid;

use crate::{error::ApiError, response::AppResponse, DynAdminsRepo};

pub fn routes(admin_repo: DynAdminsRepo) -> Router {
    Router::new()
        .route("/admins", get(get_all).post(create_one))
        .route("/admins/:id", delete(delete_one))
        .with_state(admin_repo)
}

async fn create_one(
    State(repo): State<DynAdminsRepo>,
    Json(admin): Json<CreateAdmin>,
) -> Result<Json<Admin>, ApiError> {
    Ok(Json(repo.create(admin).await?))
}

async fn get_all(State(repo): State<DynAdminsRepo>) -> Result<AppResponse<Vec<Admin>>, ApiError> {
    Ok(AppResponse::with_content(
        repo.get_all().await?,
        "retreived all admins successfully",
    ))
}

pub async fn delete_one(
    Path(id): Path<Uuid>,
    State(repo): State<DynAdminsRepo>,
) -> Result<AppResponse<()>, ApiError> {
    repo.delete_by_id(id).await?;
    Ok(AppResponse::no_content("deleted an admin successfully"))
}
