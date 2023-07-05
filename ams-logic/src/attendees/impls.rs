use std::{path::PathBuf, sync::Arc};

use sea_orm::{prelude::*, Set};
use tokio::fs;

use super::*;

pub struct AttendeesRepo {
    db: Arc<DatabaseConnection>,
    assets: PathBuf,
}

impl AsRef<DatabaseConnection> for AttendeesRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.db
    }
}

impl AttendeesRepo {
    pub fn new(db: Arc<DatabaseConnection>, assets: impl Into<PathBuf>) -> Self {
        let assets: PathBuf = assets.into();
        if assets.exists() == false {
            std::fs::create_dir_all(&assets).unwrap();
        }

        Self {
            db,
            assets: assets.into(),
        }
    }

    async fn save_image(&self, id: Uuid, image: &[u8], file_name: &str) -> PathBuf {
        let attendee_dir = self.assets.join(id.to_string());

        if attendee_dir.exists() == false {
            fs::create_dir_all(&attendee_dir).await.unwrap();
        }

        let image_path = attendee_dir.join(file_name);

        fs::write(&image_path, image).await.unwrap();

        image_path
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
            image,
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

        if let Some((image, file_name)) = image {
            let path = self.save_image(id, &image, &file_name).await;
            if file_name == "image.png" {
                attendee.image = Set(Some(path.to_string_lossy().into()));
            }
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
        let collections: Vec<Attendee> = attendees::Entity::find()
            .all(self.as_ref())
            .await?
            .into_iter()
            .map(Attendee::from)
            .collect();
        Ok(collections)
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        attendees::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await?;
        Ok(())
    }
}
