pub mod impls;
pub mod models;

use sea_orm::prelude::async_trait::async_trait;
use uuid::Uuid;

pub use impls::*;
pub use models::*;

use crate::error::RepoError;

#[async_trait]
pub trait SubjectsRepoTrait {
    async fn create(&self, subject: CreateSubject) -> Result<Subject, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Subject, RepoError>;
    async fn get(&self, filter: SubjectsFilter) -> Result<Vec<Subject>, RepoError>;
    async fn get_all_attendees(&self, id: Uuid) -> Result<Vec<Attendee>, RepoError>;
    async fn update(&self, id: Uuid, update_subject: UpdateSubject) -> Result<Subject, RepoError>;
    async fn add_attendee(&self, id: Uuid, attendee_id: Uuid) -> Result<(), RepoError>;
    async fn remove_attendee(&self, id: Uuid, attendee_id: Uuid) -> Result<(), RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
}
