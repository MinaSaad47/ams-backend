use std::borrow::Cow;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use ams_logic::prelude::*;

use crate::auth::AuthBody;

#[derive(Debug, ToSchema, Serialize)]
pub struct AttendeesList(#[schema(inline)] Vec<Instructor>);
#[derive(Debug, ToSchema, Serialize)]
pub struct InstructorsList(#[schema(inline)] Vec<Instructor>);
#[derive(Debug, ToSchema, Serialize)]
pub struct SubjectsList(#[schema(inline)] Vec<Subject>);
#[derive(Debug, ToSchema, Serialize)]
pub struct SubjectDatesList(#[schema(inline)] Vec<SubjectDate>);
#[derive(Debug, ToSchema, Serialize)]
pub struct AttendancesList(#[schema(inline)] Vec<Attendance>);

#[derive(Debug, ToSchema, Serialize, Deserialize)]
#[serde(tag = "status", rename = "success")]
#[aliases(
    AuthResponse = AppResponse<'a, AuthBody>,
    AdminResponse = AppResponse<'a, Admin>,
    InstructorResponse = AppResponse<'a, Instructor>,
    InstructorsListResponse = AppResponse<'a, InstructorsList>,
    AttendeeResponse = AppResponse<'a, Attendee>,
    AttendeesListResponse = AppResponse<'a, AttendeesList>,
    SubjectResponse = AppResponse<'a, Subject>,
    SubjectsListResponse = AppResponse<'a, SubjectsList>,
    SubjectDateResponse = AppResponse<'a, SubjectDate>,
    SubjectDatesListResponse = AppResponse<'a, SubjectDatesList>,
    AttendanceResponse = AppResponse<'a, Attendance>,
    AttendancesListResponse = AppResponse<'a, AttendancesList>
)]
pub struct AppResponse<'a, Data> {
    #[serde(skip)]
    pub code: StatusCode,
    pub message: Cow<'a, str>,
    pub data: Option<Data>,
}

pub trait AppResponseMsgExt<'a>
where
    Self: Into<Cow<'a, str>>,
{
    fn response(self) -> AppResponse<'a, ()> {
        AppResponse {
            code: StatusCode::OK,
            message: self.into(),
            data: None,
        }
    }
}

impl<'a> AppResponseMsgExt<'a> for &'a str {}
impl<'a> AppResponseMsgExt<'a> for String {}

impl<'a, T> From<T> for AppResponse<'a, ()>
where
    T: Into<Cow<'a, str>>,
{
    fn from(value: T) -> Self {
        Self {
            code: StatusCode::OK,
            message: value.into(),
            data: None,
        }
    }
}

pub trait AppResponseDataExt<'a, Msg>
where
    Self: Sized + Serialize,
    Msg: Into<Cow<'a, str>>,
{
    fn create_response(self, message: Msg) -> AppResponse<'a, Self> {
        AppResponse {
            code: StatusCode::CREATED,
            message: message.into(),
            data: Some(self),
        }
    }
    fn ok_response(self, message: Msg) -> AppResponse<'a, Self> {
        AppResponse {
            code: StatusCode::OK,
            message: message.into(),
            data: Some(self),
        }
    }
}

impl<'a, Data, Msg> AppResponseDataExt<'a, Msg> for Data
where
    Data: Sized + Serialize,
    Msg: Into<Cow<'a, str>>,
{
}

impl<'a, Data> IntoResponse for AppResponse<'a, Data>
where
    Data: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (self.code, Json(self)).into_response()
    }
}
