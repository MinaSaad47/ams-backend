use std::{path::PathBuf, sync::Arc};

use sea_orm::Set;
use tokio::fs;

use super::*;

pub struct InstructorsRepo {
    db: Arc<DatabaseConnection>,
    assets: PathBuf,
}

impl InstructorsRepo {
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

    async fn save_image(&self, id: Uuid, image: Vec<u8>) -> PathBuf {
        let instructor_dir = self.assets.join(id.to_string());

        if instructor_dir.exists() == false {
            fs::create_dir_all(&instructor_dir).await.unwrap();
        }

        let image_path = instructor_dir.join("image.png");

        fs::write(&image_path, &image).await.unwrap();

        image_path
    }
}

impl AsRef<DatabaseConnection> for InstructorsRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.db
    }
}

#[async_trait]
impl InstructorsRepoTrait for InstructorsRepo {
    async fn create(&self, instructor: CreateInstructor) -> Result<Instructor, RepoError> {
        Ok(instructors::ActiveModel {
            name: Set(instructor.name),
            email: Set(instructor.email),
            password: Set(instructor.password),
            number: Set(instructor.number),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await?
        .into())
    }
    async fn update(
        &self,
        id: Uuid,
        UpdateInstructor {
            name,
            email,
            password,
            image,
            number,
        }: UpdateInstructor,
    ) -> Result<Instructor, RepoError> {
        let mut instructor: instructors::ActiveModel = instructors::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructors".to_owned()))?
            .into();

        if let Some(name) = name {
            instructor.name = Set(name);
        }
        if let Some(email) = email {
            instructor.email = Set(email);
        }
        if let Some(password) = password {
            instructor.password = Set(password);
        }
        if let Some(number) = number {
            instructor.number = Set(number);
        }

        if let Some(image) = image {
            let path = self.save_image(id, image).await;
            instructor.image = Set(Some(path.to_string_lossy().into()))
        }

        let instructor: instructors::Model = instructor.update(self.as_ref()).await?;

        Ok(instructor.into())
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Instructor, RepoError> {
        Ok(instructors::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructors".to_owned()))?
            .into())
    }
    async fn get_by_email(&self, email: String) -> Result<Instructor, RepoError> {
        Ok(instructors::Entity::find()
            .filter(instructors::Column::Email.eq(&email))
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructors".to_owned()))?
            .into())
    }
    async fn get_all(&self) -> Result<Vec<Instructor>, RepoError> {
        let collection: Vec<Instructor> = instructors::Entity::find()
            .all(self.as_ref())
            .await?
            .into_iter()
            .map(Instructor::from)
            .collect();
        Ok(collection)
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        instructors::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await?;
        Ok(())
    }
}
