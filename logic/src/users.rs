use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, DatabaseConnection, EntityTrait, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::{sea_orm_active_enums, users};

#[async_trait]
pub trait UsersRepoTrait {
    async fn create(&self, user: CreateUser) -> User;
    async fn get_by_id(&self, id: Uuid) -> User;
    async fn get_all(&self) -> Vec<User>;
    async fn delete_by_id(&self, id: Uuid);
}

pub struct UsersRepository(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for UsersRepository {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
impl UsersRepoTrait for UsersRepository {
    async fn create(&self, user: CreateUser) -> User {
        users::ActiveModel {
            name: Set(user.name),
            email: Set(user.email),
            password: Set(user.password),
            number: Set(user.number),
            role: Set(user.role.into()),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await
        .unwrap()
        .into()
    }
    async fn get_by_id(&self, id: Uuid) -> User {
        users::Entity::find_by_id(id)
            .one(self.as_ref())
            .await
            .unwrap()
            .unwrap()
            .into()
    }
    async fn get_all(&self) -> Vec<User> {
        users::Entity::find()
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .map(User::from)
            .collect()
    }
    async fn delete_by_id(&self, id: Uuid) {
        users::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await
            .unwrap();
    }
}

#[derive(Deserialize, Serialize, Debug)]
enum UserRole {
    Instructor,
    Attendee,
}

impl From<sea_orm_active_enums::UserRole> for UserRole {
    fn from(value: sea_orm_active_enums::UserRole) -> Self {
        match value {
            sea_orm_active_enums::UserRole::Attendee => Self::Attendee,
            sea_orm_active_enums::UserRole::Instructor => Self::Instructor,
        }
    }
}

impl From<UserRole> for sea_orm_active_enums::UserRole {
    fn from(value: UserRole) -> Self {
        match value {
            UserRole::Attendee => Self::Attendee,
            UserRole::Instructor => Self::Instructor,
        }
    }
}

#[derive(Serialize)]
pub struct User {
    id: Uuid,
    number: i64,
    name: String,
    email: String,
    password: String,
    role: UserRole,
    create_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
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
