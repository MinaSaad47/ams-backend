use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};

use logic::subjects::{CreateSubject, Subject};

use uuid::Uuid;

use crate::{AppState, SubjectsRepo};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects", get(get_all).post(create_one))
        .route("/subjects/:id", delete(delete_one))
}

async fn create_one(
    State(repo): State<SubjectsRepo>,
    Json(subject): Json<CreateSubject>,
) -> Json<Subject> {
    Json(repo.create(subject).await)
}

async fn get_all(State(repo): State<SubjectsRepo>) -> Json<Vec<Subject>> {
    Json(repo.get_all().await)
}

pub async fn delete_one(Path(id): Path<Uuid>, State(repo): State<SubjectsRepo>) {
    repo.delete_by_id(id).await;
}
