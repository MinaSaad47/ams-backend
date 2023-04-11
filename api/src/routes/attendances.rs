use axum::{
    extract::{Path, State},
    routing::{get, put},
    Router,
};
use uuid::Uuid;

use logic::prelude::*;

use crate::{
    app::{self, DynAttendancesRepo, DynSubjectsRepo},
    auth::{AuthError, Claims, User},
    error::ApiError,
    response::{AppResponse, AppResponseDataExt},
};

pub(crate) fn routes() -> Router<app::State> {
    Router::new()
        .route("/attendances/subjects/:id", get(get_all_for_one_subject))
        .route("/attendances/subjects/:id/attendees/:id", put(create_one))
}

#[utoipa::path(
    get,
    path = "/attendances/subjects/{subject_id}",
    responses(
        (status = OK, body = AttendancesListResponse)
    ),
    security(("api_jwt_token" = []))
)]
#[tracing::instrument(level = "trace", skip_all, ret)]
pub async fn get_all_for_one_subject(
    State(attendances_repo): State<DynAttendancesRepo>,
    State(subjects_repo): State<DynSubjectsRepo>,
    Path(subject_id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<'static, Vec<Attendance>>, ApiError> {
    match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id)
            if Some(id) == {
                subjects_repo
                    .get_by_id(subject_id)
                    .await?
                    .instructor
                    .map(|instructor| instructor.id)
            } => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let attendances = attendances_repo
        .get(AttendancesFilter {
            subject_id: Some(subject_id),
            ..Default::default()
        })
        .await?;

    let response = attendances.ok_response("retreived all attendances for a subject successfully");

    Ok(response)
}

#[utoipa::path(
    put,
    path = "/attendances/subjects/{subject_id}/attendees/{attendee_id}",
    responses(
        (status = OK, body = AttendanceResponse)
    ),
    security(("api_jwt_token" = []))
)]
pub async fn create_one(
    State(repo): State<DynAttendancesRepo>,
    Path((subject_id, attendee_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<'static, Attendance>, ApiError> {
    match claimes.user {
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

    let respone = attendance.create_response("attendance was taken successfully");

    Ok(respone)
}
