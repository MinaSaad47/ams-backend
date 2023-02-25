use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{database::instructors, error::RepoError};

#[async_trait]
pub trait InstructorsRepoTrait {
    async fn create(&self, instructor: CreateInstructor) -> Result<Instructor, RepoError>;
    async fn update(
        &self,
        id: Uuid,
        update_instructor: UpdateInstructor,
    ) -> Result<Instructor, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Instructor, RepoError>;
    async fn get_by_email(&self, email: String) -> Result<Instructor, RepoError>;
    async fn get_all(&self) -> Result<Vec<Instructor>, RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
}

pub struct InstructorsRepo(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for InstructorsRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
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
            number,
        }: UpdateInstructor,
    ) -> Result<Instructor, RepoError> {
        let mut instructor: instructors::ActiveModel = instructors::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructor".to_owned()))?
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

        let instructor: instructors::Model = instructor.update(self.as_ref()).await?;

        Ok(instructor.into())
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Instructor, RepoError> {
        Ok(instructors::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructor".to_owned()))?
            .into())
    }
    async fn get_by_email(&self, email: String) -> Result<Instructor, RepoError> {
        Ok(instructors::Entity::find()
            .filter(instructors::Column::Email.eq(&email))
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructor".to_owned()))?
            .into())
    }
    async fn get_all(&self) -> Result<Vec<Instructor>, RepoError> {
        Ok(instructors::Entity::find()
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .map(Instructor::from)
            .collect())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        instructors::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Instructor {
    pub id: Uuid,
    pub number: i64,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<instructors::Model> for Instructor {
    fn from(
        instructors::Model {
            id,
            number,
            name,
            email,
            password,
            create_at,
            updated_at,
        }: instructors::Model,
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

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstructor {
    #[schema(example = "Mina Instructor")]
    name: String,
    #[schema(example = "MinaInstructor@outlook.com")]
    email: String,
    #[schema(example = "12345678")]
    password: String,
    #[schema(example = 13213321)]
    number: i64,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstructor {
    #[schema(example = "Emil Instructor")]
    name: Option<String>,
    #[schema(example = "EmilInstructor@outlook.com")]
    email: Option<String>,
    #[schema(example = "12345678")]
    password: Option<String>,
    #[schema(example = 3232323)]
    number: Option<i64>,
}
