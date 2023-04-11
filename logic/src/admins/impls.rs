use std::sync::Arc;

use sea_orm::{
    prelude::{async_trait::async_trait, *},
    Set,
};
use uuid::Uuid;

use crate::entity::admins;

use super::{models::*, AdminsRepoTrait};

use crate::prelude::RepoError;

pub struct AdminsRepo(pub Arc<DatabaseConnection>);

impl AsRef<DatabaseConnection> for AdminsRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[async_trait]
impl AdminsRepoTrait for AdminsRepo {
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
            .ok_or(RepoError::NotFound("admins".to_owned()))?
            .into())
    }
    async fn get_by_email(&self, email: String) -> Result<Admin, RepoError> {
        Ok(admins::Entity::find()
            .filter(admins::Column::Email.eq(&email))
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("admins".to_owned()))?
            .into())
    }
    async fn get_all(&self) -> Result<Vec<Admin>, RepoError> {
        let collection: Vec<Admin> = admins::Entity::find()
            .all(self.as_ref())
            .await?
            .into_iter()
            .map(Admin::from)
            .collect();
        if collection.is_empty() {
            Err(RepoError::NotFound("admins".to_owned()))
        } else {
            Ok(collection)
        }
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        admins::Entity::delete_by_id(id).exec(self.as_ref()).await?;
        Ok(())
    }
}
