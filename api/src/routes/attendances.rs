use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Router,
};

use logic::attendances::Attendance;

use uuid::Uuid;

use crate::{error::ApiError, response::AppResponse, DynAttendancesRepo};

pub fn routes(attendances_repo: DynAttendancesRepo) -> Router {
    Router::new()
        .route("/attendances", get(get_all))
        .route("/attendances/:id", delete(delete_one))
        .with_state(attendances_repo)
}

async fn get_all(
    State(repo): State<DynAttendancesRepo>,
) -> Result<AppResponse<Vec<Attendance>>, ApiError> {
    Ok(AppResponse::with_content(
        repo.get_all().await?,
        "retreived all attendances successfully",
    ))
}

pub async fn delete_one(
    Path(id): Path<Uuid>,
    State(repo): State<DynAttendancesRepo>,
) -> Result<AppResponse<()>, ApiError> {
    repo.delete_by_id(id).await?;
    Ok(AppResponse::no_content(
        "deleted an Attendance successfully",
    ))
}
