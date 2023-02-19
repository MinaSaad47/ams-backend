use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{database::attendees, error::RepoError};

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

pub struct AttendeesRepo(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for AttendeesRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
impl AttendeesRepoTrait for AttendeesRepo {
    async fn create(&self, attendee: CreateAttendee) -> Result<Attendee, RepoError> {
        Ok(attendees::ActiveModel {
            name: Set(attendee.name),
            email: Set(attendee.email),
            password: Set(attendee.password),
            number: Set(attendee.number),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await?
        .into())
    }
    async fn update(
        &self,
        id: Uuid,
        UpdateAttendee {
            name,
            email,
            password,
            number,
        }: UpdateAttendee,
    ) -> Result<Attendee, RepoError> {
        let mut attendee: attendees::ActiveModel = attendees::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendee".to_owned()))?
            .into();

        if let Some(name) = name {
            attendee.name = Set(name);
        }
        if let Some(email) = email {
            attendee.email = Set(email);
        }
        if let Some(password) = password {
            attendee.password = Set(password);
        }
        if let Some(number) = number {
            attendee.number = Set(number);
        }

        let attendee: attendees::Model = attendee.update(self.as_ref()).await?;

        Ok(attendee.into())
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Attendee, RepoError> {
        Ok(attendees::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendee".to_owned()))?
            .into())
    }
    async fn get_by_email(&self, email: String) -> Result<Attendee, RepoError> {
        Ok(attendees::Entity::find()
            .filter(attendees::Column::Email.eq(&email))
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendee".to_owned()))?
            .into())
    }
    async fn get_all(&self) -> Result<Vec<Attendee>, RepoError> {
        Ok(attendees::Entity::find()
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .map(Attendee::from)
            .collect())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        attendees::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await?;
        Ok(())
    }
}

#[derive(Serialize)]
pub struct Attendee {
    pub id: Uuid,
    pub number: i64,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<attendees::Model> for Attendee {
    fn from(
        attendees::Model {
            id,
            number,
            name,
            email,
            password,
            create_at,
            updated_at,
        }: attendees::Model,
    ) -> Self {
        Self {
            id,
            name,
            number,
            email,
            password,
            create_at,
            updated_at,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttendee {
    name: String,
    email: String,
    password: String,
    number: i64,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAttendee {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
    number: Option<i64>,
}
