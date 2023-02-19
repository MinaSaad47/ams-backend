use axum::{
    extract::{FromRef, Path, State},
    routing::{get, post},
    Json, Router,
};

use jsonwebtoken::{encode, Header};
use logic::{
    instructors::{CreateInstructor, Instructor, UpdateInstructor},
    subjects::Subject,
};
use uuid::Uuid;

use crate::{
    auth::{AuthBody, AuthError, AuthPayload, Claims, User, KEYS},
    error::ApiError,
    response::AppResponse,
    DynAttendancesRepo, DynInstructorsRepo, DynSubjectsRepo,
};

#[derive(FromRef, Clone)]
pub struct InstructorsState {
    pub instructors_repo: DynInstructorsRepo,
    pub subjects_repo: DynSubjectsRepo,
    pub attendances_repo: DynAttendancesRepo,
}

pub fn routes(instructors_state: InstructorsState) -> Router {
    Router::new()
        .route(
            "/instructors",
            get(get_all_instructors).post(create_one_instructor),
        )
        .route(
            "/instructors/:id",
            get(get_one_instructor)
                .patch(update_one_instructor)
                .delete(delete_one_instructor),
        )
        .route(
            "/instructors/<id>/subjects",
            post(get_all_subjects_for_one_instructor),
        )
        .route(
            "/instructors/<id>/subjects/<id>",
            get(get_one_subject_for_one_instructor)
                .put(put_one_subjects_to_one_instructor)
                .delete(delete_one_subjects_from_one_instructor),
        )
        .route(
            "/instructors/<id>/subjects/<id>",
            post(login_one_instructor),
        )
        .route("/instructors/login", post(login_one_instructor))
        .with_state(instructors_state)
}

/*
* Instructors Routes
*/

async fn get_all_instructors(
    State(repo): State<DynInstructorsRepo>,
    claimes: Claims,
) -> Result<AppResponse<Vec<Instructor>>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let instructors = repo.get_all().await?;
    let response = AppResponse::created(instructors, "retreived all instructors successfully");
    Ok(response)
}

async fn create_one_instructor(
    State(repo): State<DynInstructorsRepo>,
    claimes: Claims,
    Json(instructor): Json<CreateInstructor>,
) -> Result<AppResponse<Instructor>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let instructor = repo.create(instructor).await?;
    let response = AppResponse::created(instructor, "create on instructor successfully");

    Ok(response)
}

async fn get_one_instructor(
    State(repo): State<DynInstructorsRepo>,
    Path(id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<Instructor>, ApiError> {
    let _ = match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id) if id == id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let instructor = repo.get_by_id(id).await?;
    let response = AppResponse::with_content(instructor, "retreived an instructor successfully");

    Ok(response)
}

async fn update_one_instructor(
    State(repo): State<DynInstructorsRepo>,
    Path(id): Path<Uuid>,
    claimes: Claims,
    Json(update_instructor): Json<UpdateInstructor>,
) -> Result<AppResponse<Instructor>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let instructor = repo.update(id, update_instructor).await?;
    let response = AppResponse::with_content(instructor, "update the instructor successfully");

    Ok(response)
}

async fn delete_one_instructor(
    State(repo): State<DynInstructorsRepo>,
    Path(id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<()>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.delete_by_id(id).await?;
    let response = AppResponse::no_content("deleted one instructor successfully");

    Ok(response)
}

async fn login_one_instructor(
    State(repo): State<DynInstructorsRepo>,
    Json(payload): Json<AuthPayload>,
) -> Result<AppResponse<AuthBody>, ApiError> {
    let instructor = repo.get_by_email(payload.email).await?;

    if payload.password != instructor.password {
        return Err(AuthError::WrongCredentials.into());
    }

    let claims = Claims {
        exp: usize::max_value(),
        user: User::Instructor(instructor.id),
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

    let response =
        AppResponse::with_content(AuthBody { token }, "logged in as instructor successfully");

    Ok(response)
}

async fn get_all_subjects_for_one_instructor() -> Result<AppResponse<Vec<Subject>>, ApiError> {
    todo!()
}

async fn get_one_subject_for_one_instructor() -> Result<AppResponse<Subject>, ApiError> {
    todo!()
}

async fn put_one_subjects_to_one_instructor() -> Result<AppResponse<()>, ApiError> {
    todo!()
}

async fn delete_one_subjects_from_one_instructor() -> Result<AppResponse<()>, ApiError> {
    todo!()
}
