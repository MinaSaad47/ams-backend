use tokio::fs::File;
use utoipa::{
    openapi::{
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        Server,
    },
    Modify, OpenApi, ToSchema,
};

use crate::response::*;

#[derive(ToSchema)]
pub struct Image {
    #[schema(value_type = Option<String>, format = Binary)]
    pub image: Option<File>,
    #[schema(value_type = Option<String>, format = Binary)]
    pub any: Option<File>,
}

#[derive(ToSchema)]
pub struct Classifier {
    #[schema(value_type = String, format = Binary)]
    pub any: File,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::admins::login,

        crate::routes::config::upload_classifier,
        crate::routes::config::face_recognition,

        crate::routes::instructors::login_with_creds,
        crate::routes::instructors::login_with_token,
        crate::routes::instructors::get_all,
        crate::routes::instructors::get_one,
        crate::routes::instructors::create_one,
        crate::routes::instructors::update_one,
        crate::routes::instructors::delete_one,
        crate::routes::instructors::get_all_subjects_for_one,
        crate::routes::instructors::get_one_subject_for_one,
        crate::routes::instructors::put_one_subject_to_one,
        crate::routes::instructors::delete_one_subject_from_one,
        crate::routes::instructors::upload_image,

        crate::routes::attendees::login_with_creds,
        crate::routes::attendees::login_with_token,
        crate::routes::attendees::get_all,
        crate::routes::attendees::get_all_with_image,
        crate::routes::attendees::get_one,
        crate::routes::attendees::create_one,
        crate::routes::attendees::update_one,
        crate::routes::attendees::delete_one,
        crate::routes::attendees::get_all_subjects_for_one,
        crate::routes::attendees::get_one_subject_for_one,
        crate::routes::attendees::put_one_subject_to_one,
        crate::routes::attendees::delete_one_subject_from_one,
        crate::routes::attendees::upload_image,

        crate::routes::subjects::get_all,
        crate::routes::subjects::get_one,
        crate::routes::subjects::create_one,
        crate::routes::subjects::update_one,
        crate::routes::subjects::delete_one,
        crate::routes::subjects::get_all_attendees,
        crate::routes::subjects::add_one_subject_date,
        crate::routes::subjects::delete_one_subject_date,

        crate::routes::attendances::get_all_for_one_subject,
        crate::routes::attendances::create_one,
    ),
    components(
        schemas(
            crate::routes::config::FaceRecognition,
            crate::app::config::FaceRecModeKind,
            crate::auth::AuthPayload,
            crate::auth::AuthBody,
            ams_logic::admins::Admin,
            ams_logic::attendees::Attendee,
            ams_logic::instructors::Instructor,
            ams_logic::instructors::CreateInstructor,
            ams_logic::instructors::UpdateInstructor,
            ams_logic::attendees::CreateAttendee,
            ams_logic::attendees::UpdateAttendee,
            ams_logic::attendances::Attendance,
            ams_logic::subjects::Subject,
            ams_logic::subjects::CreateSubject,
            ams_logic::subjects::UpdateSubject,
            ams_logic::subjects::SubjectDate,
            ams_logic::subjects::CreateSubjectDate,
            AuthResponse,
            AdminResponse,
            InstructorsList,
            AttendeesList,
            SubjectsList,
            AttendancesList,
            InstructorResponse,
            InstructorsListResponse,
            AttendeeResponse,
            AttendeesListResponse,
            SubjectResponse,
            SubjectsListResponse,
            SubjectDateResponse,
            SubjectDatesListResponse,
            AttendanceResponse,
            AttendancesListResponse,
            Image,
            Classifier,
        ),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDocs;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_jwt_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
        openapi.servers = Some(vec![Server::new("/api")])
    }
}
