use axum::{http::StatusCode, response::IntoResponse, Json};
use logic::{
    admins::Admin, attendances::Attendance, attendees::Attendee, instructors::Instructor,
    subjects::Subject,
};
use sea_orm::sea_query::tests_cfg::json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::auth::AuthBody;

#[derive(Debug, ToSchema, Serialize)]
pub struct AttendeesList(#[schema(inline)] Vec<Instructor>);
#[derive(Debug, ToSchema, Serialize)]
pub struct InstructorsList(#[schema(inline)] Vec<Instructor>);
#[derive(Debug, ToSchema, Serialize)]
pub struct SubjectsList(#[schema(inline)] Vec<Subject>);
#[derive(Debug, ToSchema, Serialize)]
pub struct AttendancesList(#[schema(inline)] Vec<Attendance>);

#[derive(Debug, ToSchema)]
#[aliases(
    AuthResponse = AppResponse<AuthBody>,
    AdminResponse = AppResponse<Admin>,
    InstructorResponse = AppResponse<Instructor>,
    InstructorsListResponse = AppResponse<InstructorsList>,
    AttendeeResponse = AppResponse<Attendee>,
    AttendeesListResponse = AppResponse<AttendeesList>,
    SubjectResponse = AppResponse<Subject>,
    SubjectsListResponse = AppResponse<SubjectsList>,
    AttendanceResponse = AppResponse<Attendance>,
    AttendancesListResponse = AppResponse<AttendancesList>
)]
pub struct AppResponse<Data>
where
    Data: Serialize,
{
    #[schema(value_type = i16, example = 200)]
    pub code: StatusCode,
    pub message: String,
    pub data: Option<Data>,
}

impl<Data> AppResponse<Data>
where
    Data: Serialize,
{
    pub fn created(data: Data, message: &str) -> Self {
        Self {
            code: StatusCode::CREATED,
            message: message.to_owned(),
            data: Some(data),
        }
    }
    pub fn no_content(message: &str) -> Self {
        Self {
            code: StatusCode::OK,
            message: message.to_owned(),
            data: None,
        }
    }
    pub fn with_content(data: Data, message: &str) -> Self {
        Self {
            code: StatusCode::OK,
            message: message.to_owned(),
            data: Some(data),
        }
    }
}

impl<Data> IntoResponse for AppResponse<Data>
where
    Data: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        match self.data {
            Some(data) => {
                Json(json!({"code": self.code.as_u16(), "status": true, "message": self.message, "data": data}))
                    .into_response()
            }
            None =>  {

                Json(json!({"code": self.code.as_u16(), "status": true, "message": self.message}))
                    .into_response()
            }
        }
    }
}
