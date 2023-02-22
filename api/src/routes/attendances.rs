use axum::{
    extract::{FromRef, Path, State},
    routing::{get, put},
    Router,
};

use logic::attendances::{Attendance, AttendancesFilter, CreateAttendance};
use uuid::Uuid;

use crate::{
    auth::{AuthError, Claims, User},
    error::ApiError,
    response::AppResponse,
    DynAttendancesRepo, DynSubjectsRepo,
};

#[derive(Clone, FromRef)]
pub struct AttandancesState {
    pub attendances_repo: DynAttendancesRepo,
    pub subjects_repo: DynSubjectsRepo,
}

pub fn routes(attendances_state: AttandancesState) -> Router {
    Router::new()
        .route("/attendances/subjects/<id>", get(get_all_for_one_subject))
        .route("/attendances/subjects/<id>/attendees/<id>", put(create_one))
        .with_state(attendances_state)
}

#[utoipa::path(
    get,
    path = "/attendances/subjects/{id}",
    params(
        ("id" = Uuid, Path, description = "subject id"),
    ),
    responses(
        (status = OK, body = AttendancesListResponse)
    ),
    security(("api_jwt_token" = []))
)]
pub async fn get_all_for_one_subject(
    State(attendances_repo): State<DynAttendancesRepo>,
    State(subjects_repo): State<DynSubjectsRepo>,
    Path(id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<Vec<Attendance>>, ApiError> {
    let _ = match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id) if id == subjects_repo.get_by_id(id).await?.id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let attendances = attendances_repo
        .get(AttendancesFilter {
            subject_id: Some(id),
            ..Default::default()
        })
        .await?;

    let response = AppResponse::with_content(
        attendances,
        "retreived all attendances for a subject successfully",
    );

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/attendances/subjects/{subject_id}/attendees/{attendee_id}",
    params(
        ("subject_id" = Uuid, Path, description = "subject id"),
        ("attendee_id" = Uuid, Path, description = "attendee id"),
    ),
    responses(
        (status = OK, body = AttendanceResponse)
    ),
    security(("api_jwt_token" = []))
)]
pub async fn create_one(
    State(repo): State<DynAttendancesRepo>,
    Path((subject_id, attendee_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<Attendance>, ApiError> {
    let _ = match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id) if id == repo.get_by_id(id).await?.id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let attendance = repo
        .create(CreateAttendance {
            subject_id,
            attendee_id,
        })
        .await?;

    let respone = AppResponse::with_content(attendance, "attendance was taken successfully");

    Ok(respone)
}
