use std::{net::SocketAddr, sync::Arc};

use axum::Router;

mod auth;
mod error;
mod openapi_doc;
mod response;
mod routes;

use dotenvy_macro::dotenv;
use logic::{
    admins::{AdminsRepo, AdminsRepoTrait},
    attendances::{AttendancesRepo, AttendancesRepoTrait},
    attendees::{AttendeesRepo, AttendeesRepoTrait},
    instructors::{InstructorsRepo, InstructorsRepoTrait},
    subjects::{SubjectsRepoTrait, SubjectsRepository},
};
use openapi_doc::ApiDoc;
use routes::{
    admins::{self, AdminsState},
    attendances,
    attendees::{self, AttendeesState},
    instructors::{self, InstructorsState},
    subjects,
};
use sea_orm::Database;
use tower_http::trace::TraceLayer;

use dotenvy::dotenv;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub type DynAdminsRepo = Arc<dyn AdminsRepoTrait + Send + Sync>;
pub type DynInstructorsRepo = Arc<dyn InstructorsRepoTrait + Send + Sync>;
pub type DynAttendeesRepo = Arc<dyn AttendeesRepoTrait + Send + Sync>;
pub type DynAttendancesRepo = Arc<dyn AttendancesRepoTrait + Send + Sync>;
pub type DynSubjectsRepo = Arc<dyn SubjectsRepoTrait + Send + Sync>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    dotenv().ok();

    let db = Database::connect(dotenv!("DATABASE_URL"))
        .await
        .expect("posgresql connection");

    let admins_repo = Arc::new(AdminsRepo(db.clone()));
    let instructors_repo = Arc::new(InstructorsRepo(db.clone()));
    let attendees_repo = Arc::new(AttendeesRepo(db.clone()));
    let subjects_repo = Arc::new(SubjectsRepository(db.clone()));
    let attendances_repo = Arc::new(AttendancesRepo(db));

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .nest(
            "/api",
            Router::new()
                .merge(admins::routes(AdminsState {
                    admins_repo: admins_repo.clone(),
                }))
                .merge(instructors::routes(InstructorsState {
                    instructors_repo: instructors_repo.clone(),
                    subjects_repo: subjects_repo.clone(),
                    attendances_repo: attendances_repo.clone(),
                }))
                .merge(attendances::routes(attendances::AttandancesState {
                    attendances_repo: attendances_repo.clone(),
                    subjects_repo: subjects_repo.clone(),
                }))
                .merge(attendees::routes(AttendeesState {
                    attendees_repo: attendees_repo.clone(),
                    subjects_repo: subjects_repo.clone(),
                    attedances_repo: attendances_repo.clone(),
                }))
                .merge(subjects::routes(subjects_repo.clone())),
        )
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("axum server");
}
