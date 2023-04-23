mod impls;
mod models;

use sea_orm::prelude::{async_trait::async_trait, *};

pub use impls::*;
pub use models::*;

use crate::error::RepoError;

use crate::entity::instructors;

#[async_trait]
pub trait InstructorsRepoTrait {
    async fn create(&self, instructor: CreateInstructor) -> Result<Instructor, RepoError>;
    async fn update(
        &self,
        id: Uuid,
        update_instructor: UpdateInstructor,
    ) -> Result<Instructor, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Instructor, RepoError>;
    async fn get_by_email(&self, email: String) -> Result<Instructor, RepoError>;
    async fn get_all(&self) -> Result<Vec<Instructor>, RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
}
