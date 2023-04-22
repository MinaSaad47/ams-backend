use axum::{
    extract::{Path, State},
    routing::{delete, get, put},
    Json, Router,
};
use uuid::Uuid;

use logic::prelude::*;

use crate::{
    app::{self, DynAttendancesRepo, DynSubjectsRepo},
    auth::{AuthError, Claims, User},
    error::ApiError,
    response::{AppResponse, AppResponseDataExt, AppResponseMsgExt},
};

pub(crate) fn routes() -> Router<app::State> {
    Router::new()
        .route(
            "/attendances/subjects/:id",
            get(get_all_for_one_subject).post(create_many),
        )
        .route("/attendances/subjects/:id/attendees/:id", put(create_one))
        .route("/attendances/:id", delete(delete_one))
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
    State(attendances_repo): State<DynAttendancesRepo>,
    State(subjects_repo): State<DynSubjectsRepo>,
    Path((subject_id, attendee_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<'static, Attendance>, ApiError> {
    match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id)
            if Some(id)
                == subjects_repo
                    .get_by_id(subject_id)
                    .await?
                    .instructor
                    .map(|instructor| instructor.id) => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let attendance = attendances_repo
        .create_one(CreateAttendance {
            subject_id,
            attendee_id,
        })
        .await?;

    let respone = attendance.create_response("attendance was taken successfully");

    Ok(respone)
}

pub async fn create_many(
    State(attendances_repo): State<DynAttendancesRepo>,
    State(subjects_repo): State<DynSubjectsRepo>,
    Path(subject_id): Path<Uuid>,
    claimes: Claims,
    Json(CreateAttendances {
        attendee_ids: attendees,
    }): Json<CreateAttendances>,
) -> Result<AppResponse<'static, Vec<Attendance>>, ApiError> {
    match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id)
            if Some(id)
                == subjects_repo
                    .get_by_id(subject_id)
                    .await?
                    .instructor
                    .map(|instructor| instructor.id) => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let attendances = attendances_repo.create_many(subject_id, attendees).await?;

    let respone = attendances.create_response("attendance was taken successfully");

    Ok(respone)
}

async fn delete_one(
    State(attendances_repo): State<DynAttendancesRepo>,
    Path(attendance_id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<'static, ()>, ApiError> {
    let response = "deleted one attendee successfully".response();

    match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id)
            if Some(id)
                == attendances_repo
                    .get_by_id(attendance_id)
                    .await?
                    .subject
                    .instructor
                    .map(|instuctor| instuctor.id) => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    attendances_repo.delete_by_id(attendance_id).await?;

    Ok(response)
}
