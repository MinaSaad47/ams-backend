use std::{net::SocketAddr, sync::Arc};

use axum::Router;

mod auth;
mod error;
mod response;
mod routes;

use dotenvy_macro::dotenv;
use logic::{
    admins::{AdminsRepoTrait, AdminsRepository},
    attendances::{AttendancesRepoTrait, AttendancesRepository},
    subjects::{SubjectsRepoTrait, SubjectsRepository},
    users::{UsersRepo, UsersRepoTrait},
};
use routes::{
    admins, attendances,
    attendees::{self, AttendeesState},
    auths,
    instructors::{self, InstructorsState},
    subjects,
};
use sea_orm::Database;
use tower_http::trace::TraceLayer;

use dotenvy::dotenv;

pub type DynAdminsRepo = Arc<dyn AdminsRepoTrait + Send + Sync>;
pub type DynUsersRepo = Arc<dyn UsersRepoTrait + Send + Sync>;
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

    let admins_repo = Arc::new(AdminsRepository(db.clone()));
    let users_repo = Arc::new(UsersRepo(db.clone()));
    let subjects_repo = Arc::new(SubjectsRepository(db.clone()));
    let attendances_repo = Arc::new(AttendancesRepository(db));

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(auths::routes(users_repo.clone()))
                .merge(admins::routes(admins_repo))
                .merge(instructors::routes(InstructorsState {
                    users_repo: users_repo.clone(),
                    subjects_repo: subjects_repo.clone(),
                    attendances_repo: attendances_repo.clone(),
                }))
                .merge(attendances::routes(attendances_repo))
                .merge(attendees::routes(AttendeesState {
                    user_repo: users_repo.clone(),
                    subjects_repo: subjects_repo.clone(),
                }))
                .merge(subjects::routes(subjects_repo)),
        )
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("axum server");
}
