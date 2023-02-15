use std::{net::SocketAddr, sync::Arc};

use axum::{extract::FromRef, Router};

mod database;
mod routes;

use dotenvy_macro::dotenv;
use logic::{
    admins::{AdminsRepoTrait, AdminsRepository},
    attendances::{AttendancesRepoTrait, AttendancesRepository},
    subjects::{SubjectsRepoTrait, SubjectsRepository},
    users::{UsersRepoTrait, UsersRepository},
};
use routes::{admins, attendances, subjects, users};
use sea_orm::Database;
use tower_http::trace::TraceLayer;

use dotenvy::dotenv;

pub type AdminsRepo = Arc<dyn AdminsRepoTrait + Send + Sync>;
pub type UsersRepo = Arc<dyn UsersRepoTrait + Send + Sync>;
pub type AttendancesRepo = Arc<dyn AttendancesRepoTrait + Send + Sync>;
pub type SubjectsRepo = Arc<dyn SubjectsRepoTrait + Send + Sync>;

#[derive(FromRef, Clone)]
pub struct AppState {
    admins_repo: AdminsRepo,
    users_repo: UsersRepo,
    subjects_repo: SubjectsRepo,
    attendances_repo: AttendancesRepo,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    dotenv().ok();

    let db = Database::connect(dotenv!("DATABASE_URL")).await.unwrap();

    let admins_repo = Arc::new(AdminsRepository(db.clone()));
    let users_repo = Arc::new(UsersRepository(db.clone()));
    let subjects_repo = Arc::new(SubjectsRepository(db.clone()));
    let attendances_repo = Arc::new(AttendancesRepository(db));

    let app_state = AppState {
        admins_repo,
        users_repo,
        subjects_repo,
        attendances_repo,
    };

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(admins::routes())
                .merge(users::routes())
                .merge(attendances::routes())
                .merge(subjects::routes())
                .with_state(app_state),
        )
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
