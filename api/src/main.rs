mod auth;
mod error;
mod openapi_doc;
mod response;
mod routes;
mod test_utils;

use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use dotenvy::dotenv;
use dotenvy_macro::dotenv;
use openapi_doc::ApiDoc;
use routes::subjects::SubjectsState;
use sea_orm::{ConnectOptions, Database};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    normalize_path::NormalizePathLayer,
    trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{log, metadata};
use tracing_subscriber::{filter, prelude::*};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::{
    admins::{self, AdminsState},
    attendances,
    attendees::{self, AttendeesState},
    instructors::{self, InstructorsState},
    subjects,
};

use logic::prelude::*;

pub type DynAdminsRepo = Arc<dyn AdminsRepoTrait + Send + Sync>;
pub type DynInstructorsRepo = Arc<dyn InstructorsRepoTrait + Send + Sync>;
pub type DynAttendeesRepo = Arc<dyn AttendeesRepoTrait + Send + Sync>;
pub type DynAttendancesRepo = Arc<dyn AttendancesRepoTrait + Send + Sync>;
pub type DynSubjectsRepo = Arc<dyn SubjectsRepoTrait + Send + Sync>;

#[tokio::main]
async fn main() {
    // load envirnment variables from .env
    dotenv().ok();

    // enable tracing
    setup_tracing(metadata::LevelFilter::DEBUG);

    let db = {
        let mut opt = ConnectOptions::new(dotenv!("DATABASE_URL").to_owned());
        opt.sqlx_logging_level(log::LevelFilter::Debug);
        Arc::new(Database::connect(opt).await.expect("posgresql connection"))
    };

    let admins_repo = Arc::new(AdminsRepoPg(db.clone()));
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
                    attendances_repo: attendances_repo.clone(),
                }))
                .merge(subjects::routes(SubjectsState {
                    subjects_repo: subjects_repo.clone(),
                    attendees_repo: attendees_repo.clone(),
                })),
        )
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                        .on_failure(DefaultOnFailure::new().level(tracing::Level::INFO)),
                )
                .layer(NormalizePathLayer::trim_trailing_slash())
                .layer(CompressionLayer::new()),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("axum server");
}

fn setup_tracing(level: metadata::LevelFilter) {
    let layer1 = tracing_subscriber::fmt::layer()
        .pretty()
        .with_test_writer()
        .with_filter(filter::filter_fn(|meta| {
            meta.target().contains("api") || meta.target().contains("logic")
        }))
        .with_filter(level);

    let layer2 = tracing_subscriber::fmt::layer()
        .pretty()
        .with_test_writer()
        .with_line_number(false)
        .with_file(false)
        .with_thread_names(false)
        .with_target(false)
        .with_filter(filter::filter_fn(|meta| {
            meta.target().contains("sea_orm")
                || meta.target().contains("sqlx")
                || meta.target().contains("tower_http")
        }))
        .with_filter(level);

    tracing_subscriber::registry()
        .with(layer1)
        .with(layer2)
        .init();
}
