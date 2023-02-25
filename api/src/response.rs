use axum::{http::StatusCode, response::IntoResponse, Json};
use sea_orm::sea_query::tests_cfg::json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use logic::prelude::*;

use crate::auth::AuthBody;

#[derive(Debug, ToSchema, Serialize)]
pub struct AttendeesList(#[schema(inline)] Vec<Instructor>);
#[derive(Debug, ToSchema, Serialize)]
pub struct InstructorsList(#[schema(inline)] Vec<Instructor>);
#[derive(Debug, ToSchema, Serialize)]
pub struct SubjectsList(#[schema(inline)] Vec<Subject>);
#[derive(Debug, ToSchema, Serialize)]
pub struct AttendancesList(#[schema(inline)] Vec<Attendance>);

#[derive(Debug, ToSchema, Serialize, Deserialize)]
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
pub struct AppResponse<Data> {
    #[schema(value_type = i16, example = 200)]
    pub code: u16,
    pub message: String,
    pub data: Option<Data>,
}

impl<Data> AppResponse<Data>
where
    Data: Serialize,
{
    pub fn created(data: Data, message: &str) -> Self {
        Self {
            code: StatusCode::CREATED.into(),
            message: message.to_owned(),
            data: Some(data),
        }
    }
    pub fn no_content(message: &str) -> Self {
        Self {
            code: StatusCode::OK.into(),
            message: message.to_owned(),
            data: None,
        }
    }
    pub fn with_content(data: Data, message: &str) -> Self {
        Self {
            code: StatusCode::OK.into(),
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
            Some(data) => Json(
                json!({"code": self.code, "status": true, "message": self.message, "data": data}),
            )
            .into_response(),
            None => Json(json!({"code": self.code, "status": true, "message": self.message}))
                .into_response(),
        }
    }
}
