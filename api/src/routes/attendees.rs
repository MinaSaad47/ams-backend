use axum::{
    extract::{FromRef, Path, State},
    routing::get,
    Json, Router,
};

use logic::users::{CreateUser, User};

use uuid::Uuid;

use crate::{
    auth::Instructor, error::ApiError, response::AppResponse, DynSubjectsRepo, DynUsersRepo,
};

#[derive(Clone, FromRef)]
pub struct AttendeesState {
    pub user_repo: DynUsersRepo,
    pub subjects_repo: DynSubjectsRepo,
}

pub fn routes(attendees_state: AttendeesState) -> Router {
    Router::new()
        .route("/users", get(get_all).post(create_one))
        .route("/users/:id", get(get_one).delete(delete_one))
        .with_state(attendees_state)
}

/*
* Attendees Route
*/

async fn create_one(
    State(repo): State<DynUsersRepo>,
    Json(user): Json<CreateUser>,
) -> Result<Json<User>, ApiError> {
    Ok(Json(repo.create(user).await?))
}

async fn get_all(
    State(repo): State<DynUsersRepo>,
    _instructor: Instructor,
) -> Result<AppResponse<Vec<User>>, ApiError> {
    Ok(AppResponse::with_content(
        repo.get_all().await?,
        "retreived all users successfully",
    ))
}

async fn get_one(
    Path(id): Path<Uuid>,
    State(repo): State<DynUsersRepo>,
) -> Result<AppResponse<User>, ApiError> {
    Ok(AppResponse::with_content(
        repo.get_by_id(id).await?,
        "retreived all users successfully",
    ))
}

/*
* Subjects Routes
*/

pub async fn delete_one(
    Path(id): Path<Uuid>,
    State(repo): State<DynUsersRepo>,
) -> Result<AppResponse<()>, ApiError> {
    repo.delete_by_id(id).await?;
    Ok(AppResponse::no_content("deleted a user successfully"))
}
