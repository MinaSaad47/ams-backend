use axum::{
    extract::{FromRef, Path, State},
    routing::get,
    Json, Router,
};

use logic::{
    subjects::{CreateSubject, Subject},
    users::{CreateUser, User},
};

use uuid::Uuid;

use crate::{
    auth::{AuthError, Instructor},
    error::ApiError,
    response::AppResponse,
    DynAttendancesRepo, DynSubjectsRepo, DynUsersRepo,
};

#[derive(FromRef, Clone)]
pub struct InstructorsState {
    pub users_repo: DynUsersRepo,
    pub subjects_repo: DynSubjectsRepo,
    pub attendances_repo: DynAttendancesRepo,
}

pub fn routes(instructors_state: InstructorsState) -> Router {
    Router::new()
        .route(
            "/instructors",
            get(get_all_instrutors).post(create_one_instructor),
        )
        .route(
            "/instructors/:id",
            get(get_one_instrutor).delete(delete_one_instrutor),
        )
        .route(
            "/instructors/:id/subjects",
            get(get_all_subjects).post(create_one_subject),
        )
        .route(
            "/instructors/:id/subjects/:<id>",
            get(get_one_subject).delete(delete_one_subject),
        )
        .with_state(instructors_state)
}

/*
* Instructors Routes
*/

async fn create_one_instructor(
    State(repo): State<DynUsersRepo>,
    Json(user): Json<CreateUser>,
) -> Result<Json<User>, ApiError> {
    Ok(Json(repo.create(user).await?))
}

async fn get_all_instrutors(
    State(repo): State<DynUsersRepo>,
    _instructor: Instructor,
) -> Result<AppResponse<Vec<User>>, ApiError> {
    Ok(AppResponse::with_content(
        repo.get_all().await?,
        "retreived all users successfully",
    ))
}

async fn get_one_instrutor(
    Path(id): Path<Uuid>,
    State(repo): State<DynUsersRepo>,
) -> Result<AppResponse<User>, ApiError> {
    Ok(AppResponse::with_content(
        repo.get_by_id(id).await?,
        "retreived all users successfully",
    ))
}

pub async fn delete_one_instrutor(
    Path(id): Path<Uuid>,
    State(repo): State<DynUsersRepo>,
) -> Result<AppResponse<()>, ApiError> {
    repo.delete_by_id(id).await?;
    Ok(AppResponse::no_content("deleted a user successfully"))
}

/*
* Subjects Routes
*/
pub async fn get_all_subjects(
    State(subjects_repo): State<DynSubjectsRepo>,
    instrutor: Instructor,
) -> Result<AppResponse<Vec<Subject>>, ApiError> {
    Ok(AppResponse::with_content(
        subjects_repo.get(None, Some(instrutor.id)).await?,
        &format!("retreived all subjects for {} successfully", instrutor.name),
    ))
}

pub async fn get_one_subject(
    Path(id): Path<Uuid>,
    State(subjects_repo): State<DynSubjectsRepo>,
    instrutor: Instructor,
) -> Result<AppResponse<Subject>, ApiError> {
    let subject = subjects_repo.get_by_id(id).await?;

    if subject.instructor.id != instrutor.id {
        Err(AuthError::UnauthorizedAccess)?
    }

    let subject_name = subject.name.clone();
    Ok(AppResponse::with_content(
        subject,
        &format!(
            "retreived subject {} for {} successfully",
            subject_name, instrutor.name
        ),
    ))
}

pub async fn create_one_subject(
    State(subjects_repo): State<DynSubjectsRepo>,
    instrutor: Instructor,
    Json(mut subject): Json<CreateSubject>,
) -> Result<AppResponse<Subject>, ApiError> {
    subject.instructor_id = instrutor.id;
    let subject_name = subject.name.clone();
    Ok(AppResponse::created(
        subjects_repo.create(subject).await?,
        &format!(
            "created subject `{}` for instrutor `{}`",
            subject_name, instrutor.name
        ),
    ))
}

pub async fn delete_one_subject(
    Path(id): Path<Uuid>,
    State(subjects_repo): State<DynSubjectsRepo>,
    instrutor: Instructor,
) -> Result<AppResponse<()>, ApiError> {
    let subject = subjects_repo.get_by_id(id).await?;

    if subject.instructor.id != instrutor.id {
        Err(AuthError::UnauthorizedAccess)?
    }

    subjects_repo.delete_by_id(subject.id).await?;

    Ok(AppResponse::no_content("deleted one subject successfully"))
}
