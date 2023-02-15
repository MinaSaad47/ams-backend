use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};

use logic::admins::{Admin, CreateAdmin};

use uuid::Uuid;

use crate::{AdminsRepo, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admins", get(get_all).post(create_one))
        .route("/admins/:id", delete(delete_one))
}

async fn create_one(State(repo): State<AdminsRepo>, Json(admin): Json<CreateAdmin>) -> Json<Admin> {
    Json(repo.create(admin).await)
}

async fn get_all(State(repo): State<AdminsRepo>) -> Json<Vec<Admin>> {
    Json(repo.get_all().await)
}

pub async fn delete_one(Path(id): Path<Uuid>, State(repo): State<AdminsRepo>) {
    repo.delete_by_id(id).await
}
