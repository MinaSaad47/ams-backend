use std::sync::Arc;

use sea_orm::{prelude::*, Set};

use super::*;

pub struct AttendeesRepo(pub Arc<DatabaseConnection>);

impl AsRef<DatabaseConnection> for AttendeesRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

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
            embedding,
        }: UpdateAttendee,
    ) -> Result<Attendee, RepoError> {
        let mut attendee: attendees::ActiveModel = attendees::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendees".to_owned()))?
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
        if let Some(embedding) = embedding {
            attendee.embedding = Set(embedding);
        }

        let attendee: attendees::Model = attendee.update(self.as_ref()).await?;

        Ok(attendee.into())
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Attendee, RepoError> {
        Ok(attendees::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendees".to_owned()))?
            .into())
    }
    async fn get_by_email(&self, email: String) -> Result<Attendee, RepoError> {
        Ok(attendees::Entity::find()
            .filter(attendees::Column::Email.eq(&email))
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendees".to_owned()))?
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
