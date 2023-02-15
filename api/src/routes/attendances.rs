use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};


use logic::attendances::Attendance;

use uuid::Uuid;

use crate::{
    AppState, AttendancesRepo,
};


async fn get_all(State(repo): State<AttendancesRepo>) -> Json<Vec<Attendance>> {
    Json(repo.get_all().await)
}

pub async fn delete_one(Path(id): Path<Uuid>, State(repo): State<AttendancesRepo>) {
    repo.delete_by_id(id).await;
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/attendances", get(get_all))
        .route("/attendances/:id", delete(delete_one))
}