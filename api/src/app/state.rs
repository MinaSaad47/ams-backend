use std::sync::Arc;

use axum::extract::FromRef;
use logic::subjects::{
    AdminsRepo, AdminsRepoTrait, AttendancesRepo, AttendancesRepoTrait, AttendeesRepo,
    AttendeesRepoTrait, InstructorsRepo, InstructorsRepoTrait, SubjectsRepoTrait,
    SubjectsRepository,
};
use sea_orm::DatabaseConnection;

pub(crate) type DynAdminsRepo = Arc<dyn AdminsRepoTrait + Send + Sync>;
pub(crate) type DynInstructorsRepo = Arc<dyn InstructorsRepoTrait + Send + Sync>;
pub(crate) type DynAttendeesRepo = Arc<dyn AttendeesRepoTrait + Send + Sync>;
pub(crate) type DynAttendancesRepo = Arc<dyn AttendancesRepoTrait + Send + Sync>;
pub(crate) type DynSubjectsRepo = Arc<dyn SubjectsRepoTrait + Send + Sync>;

#[derive(FromRef, Clone)]
pub(crate) struct State {
    attendees_repo: DynAttendeesRepo,
    instructors_repo: DynInstructorsRepo,
    admins_repo: DynAdminsRepo,
    subjects_repo: DynSubjectsRepo,
    attendances_repo: DynAttendancesRepo,
}

impl State {
    pub(crate) fn new(db: DatabaseConnection) -> Self {
        let db = Arc::new(db);
        let attendees_repo = Arc::new(AttendeesRepo(db.clone()));
        let instructors_repo = Arc::new(InstructorsRepo(db.clone()));
        let admins_repo = Arc::new(AdminsRepo(db.clone()));
        let subjects_repo = Arc::new(SubjectsRepository(db.clone()));
        let attendances_repo = Arc::new(AttendancesRepo(db.clone()));
        Self {
            attendees_repo,
            instructors_repo,
            admins_repo,
            subjects_repo,
            attendances_repo,
        }
    }
}
