use chrono::{DateTime, FixedOffset};
use sea_orm::ColumnTrait;
use sea_orm::QueryFilter;
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, DatabaseConnection, EntityTrait, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{database::admins, error::RepoError};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AdminsRepoTrait {
    async fn create(&self, admin: CreateAdmin) -> Result<Admin, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Admin, RepoError>;
    async fn get_by_email(&self, email: String) -> Result<Admin, RepoError>;
    async fn get_all(&self) -> Result<Vec<Admin>, RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
}

pub struct AdminsRepository(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for AdminsRepository {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[async_trait]
impl AdminsRepoTrait for AdminsRepository {
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

#[derive(Serialize)]
pub struct Admin {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<admins::Model> for Admin {
    fn from(
        admins::Model {
            id,
            name,
            email,
            password,
            create_at,
            updated_at,
        }: admins::Model,
    ) -> Self {
        Self {
            id,
            name,
            email,
            password,
            create_at,
            updated_at,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateAdmin {
    name: String,
    email: String,
    password: String,
}
