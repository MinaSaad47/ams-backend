mod impls;
mod models;

use sea_orm::prelude::async_trait::async_trait;
use uuid::Uuid;

use crate::error::RepoError;

pub use impls::*;
pub use models::*;

#[async_trait]
pub trait AdminsRepoTrait {
    async fn create(&self, admin: CreateAdmin) -> Result<Admin, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Admin, RepoError>;
    async fn get_by_email(&self, email: String) -> Result<Admin, RepoError>;
    async fn get_all(&self) -> Result<Vec<Admin>, RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
}
