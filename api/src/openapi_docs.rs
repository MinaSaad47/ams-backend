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
    #[schema(value_type = String, format = Binary)]
    pub image: File,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::admins::login,

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

        crate::routes::attendances::get_all_for_one_subject,
        crate::routes::attendances::create_one,
    ),
    components(
        schemas(
            crate::auth::AuthPayload,
            crate::auth::AuthBody,
            logic::admins::Admin,
            logic::attendees::Attendee,
            logic::instructors::Instructor,
            logic::instructors::CreateInstructor,
            logic::instructors::UpdateInstructor,
            logic::attendees::CreateAttendee,
            logic::attendees::UpdateAttendee,
            logic::attendances::Attendance,
            logic::subjects::Subject,
            logic::subjects::CreateSubject,
            logic::subjects::UpdateSubject,
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
            AttendanceResponse,
            AttendancesListResponse,
            Image,
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
