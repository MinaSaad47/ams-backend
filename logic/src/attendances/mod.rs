mod impls;
mod models;

pub use impls::*;
pub use models::*;

use sea_orm::prelude::{async_trait::async_trait, *};

use crate::prelude::*;

#[async_trait]
pub trait AttendancesRepoTrait {
    async fn create_one(&self, attendance: CreateAttendance) -> Result<Attendance, RepoError>;
    async fn create_many(
        &self,
        subject_id: Uuid,
        attendee_ids: Vec<Uuid>,
    ) -> Result<Vec<Attendance>, RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
    async fn get(&self, attendaces_filter: AttendancesFilter)
        -> Result<Vec<Attendance>, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Attendance, RepoError>;
}
