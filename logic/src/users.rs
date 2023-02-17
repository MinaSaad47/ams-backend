use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{sea_orm_active_enums, users},
    error::RepoError,
};

#[async_trait]
pub trait UsersRepoTrait {
    async fn create(&self, user: CreateUser) -> Result<User, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<User, RepoError>;
    async fn get_by_email(&self, email: String) -> Result<User, RepoError>;
    async fn get_all(&self) -> Result<Vec<User>, RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
}

pub struct UsersRepo(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for UsersRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
impl UsersRepoTrait for UsersRepo {
    async fn create(&self, user: CreateUser) -> Result<User, RepoError> {
        Ok(users::ActiveModel {
            name: Set(user.name),
            email: Set(user.email),
            password: Set(user.password),
            number: Set(user.number),
            role: Set(user.role.into()),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await?
        .into())
    }
    async fn get_by_id(&self, id: Uuid) -> Result<User, RepoError> {
        Ok(users::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("user".to_owned()))?
            .into())
    }
    async fn get_by_email(&self, email: String) -> Result<User, RepoError> {
        Ok(users::Entity::find()
            .filter(users::Column::Email.eq(&email))
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("user".to_owned()))?
            .into())
    }
    async fn get_all(&self) -> Result<Vec<User>, RepoError> {
        Ok(users::Entity::find()
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .map(User::from)
            .collect())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        users::Entity::delete_by_id(id).exec(self.as_ref()).await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum UserRole {
    Instructor,
    Attendee,
}

impl From<sea_orm_active_enums::UserRole> for UserRole {
    fn from(value: sea_orm_active_enums::UserRole) -> Self {
        match value {
            sea_orm_active_enums::UserRole::Attendee => UserRole::Attendee,
            sea_orm_active_enums::UserRole::Instructor => UserRole::Instructor,
        }
    }
}

impl From<UserRole> for sea_orm_active_enums::UserRole {
    fn from(value: UserRole) -> Self {
        match value {
            UserRole::Attendee => sea_orm_active_enums::UserRole::Attendee,
            UserRole::Instructor => sea_orm_active_enums::UserRole::Instructor,
        }
    }
}

#[derive(Serialize)]
pub struct User {
    pub id: Uuid,
    pub number: i64,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub role: UserRole,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<users::Model> for User {
    fn from(
        users::Model {
            id,
            number,
            name,
            email,
            password,
            role,
            create_at,
            updated_at,
        }: users::Model,
    ) -> Self {
        Self {
            id,
            name,
            number,
            email,
            role: role.into(),
            password,
            create_at,
            updated_at,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateUser {
    name: String,
    email: String,
    password: String,
    number: i64,
    role: UserRole,
}
