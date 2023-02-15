use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};

use logic::users::{CreateUser, User};

use uuid::Uuid;

use crate::{AppState, UsersRepo};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/users", get(get_all).post(create_one))
        .route("/users/:id", delete(delete_one))
}

async fn create_one(State(repo): State<UsersRepo>, Json(user): Json<CreateUser>) -> Json<User> {
    Json(repo.create(user).await)
}

async fn get_all(State(repo): State<UsersRepo>) -> Json<Vec<User>> {
    Json(repo.get_all().await)
}

pub async fn delete_one(Path(id): Path<Uuid>, State(repo): State<UsersRepo>) {
    repo.delete_by_id(id).await
}
