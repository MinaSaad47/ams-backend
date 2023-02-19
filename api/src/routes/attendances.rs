use axum::{
    routing::{get, put},
    Router,
};

use logic::attendances::Attendance;

use crate::{error::ApiError, response::AppResponse, DynAttendancesRepo};

pub fn routes(attendances_repo: DynAttendancesRepo) -> Router {
    Router::new()
        .route(
            "/attendances/subjects/<id>",
            get(get_all_attendances_for_one_subject),
        )
        .route(
            "/attendances/subjects/<id>/attendees/<id>",
            put(create_one_attendance_for_one_subject_and_one_attendee),
        )
        .with_state(attendances_repo)
}

pub async fn get_all_attendances_for_one_subject() -> Result<AppResponse<Vec<Attendance>>, ApiError>
{
    todo!()
}

pub async fn create_one_attendance_for_one_subject_and_one_attendee(
) -> Result<AppResponse<Attendance>, ApiError> {
    todo!()
}
