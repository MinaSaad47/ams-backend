use sea_orm::{
    prelude::{async_trait::async_trait, *},
    Set,
};
use uuid::Uuid;

use crate::{entity::admins, prelude::RepoError};

use super::{models::*, AdminsRepoTrait};

pub struct AdminsRepoPg(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for AdminsRepoPg {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[async_trait]
impl AdminsRepoTrait for AdminsRepoPg {
    async fn create(&self, admin: CreateAdmin) -> Result<Admin, RepoError> {
        Ok(admins::ActiveModel {
            name: Set(admin.name),
            email: Set(admin.email),
            password: Set(admin.password),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await?
        .into())
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Admin, RepoError> {
        Ok(admins::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("admin".to_owned()))?
            .into())
    }
    async fn get_by_email(&self, email: String) -> Result<Admin, RepoError> {
        Ok(admins::Entity::find()
            .filter(admins::Column::Email.eq(&email))
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("user".to_owned()))?
            .into())
    }
    async fn get_all(&self) -> Result<Vec<Admin>, RepoError> {
        Ok(admins::Entity::find()
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .map(Admin::from)
            .collect())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        admins::Entity::delete_by_id(id).exec(self.as_ref()).await?;
        Ok(())
    }
}
