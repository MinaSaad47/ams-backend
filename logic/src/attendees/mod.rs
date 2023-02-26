pub mod impls;
pub mod models;

use sea_orm::prelude::async_trait::async_trait;
use uuid::Uuid;

pub use impls::*;
pub use models::*;

use crate::error::RepoError;

use crate::entity::attendees;

#[async_trait]
pub trait AttendeesRepoTrait {
    async fn create(&self, attendee: CreateAttendee) -> Result<Attendee, RepoError>;
    async fn update(
        &self,
        id: Uuid,
        update_attendee: UpdateAttendee,
    ) -> Result<Attendee, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Attendee, RepoError>;
    async fn get_by_email(&self, email: String) -> Result<Attendee, RepoError>;
    async fn get_all(&self) -> Result<Vec<Attendee>, RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
}
