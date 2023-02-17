use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, LoaderTrait, ModelTrait, QueryFilter, QueryTrait, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{subjects, users},
    error::RepoError,
    users::User,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SubjectsRepoTrait {
    async fn create(&self, subject: CreateSubject) -> Result<Subject, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Subject, RepoError>;
    async fn get(
        &self,
        name: Option<String>,
        instructor: Option<Uuid>,
    ) -> Result<Vec<Subject>, RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
}

pub struct SubjectsRepository(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for SubjectsRepository {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[async_trait]
impl SubjectsRepoTrait for SubjectsRepository {
    async fn create(&self, subject: CreateSubject) -> Result<Subject, RepoError> {
        let subject = subjects::ActiveModel {
            name: Set(subject.name),
            instructor_id: Set(subject.instructor_id),
            cron_expr: Set(subject.cron_expr),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await?;

        Ok(self.get_by_id(subject.id).await?)
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Subject, RepoError> {
        let subject = subjects::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("subject".to_owned()))?;

        let instructor = subject
            .find_related(users::Entity)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("instructor".to_owned()))?;

        Ok((subject, instructor.into()).into())
    }
    async fn get(
        &self,
        name: Option<String>,
        instructor: Option<Uuid>,
    ) -> Result<Vec<Subject>, RepoError> {
        let subjects: Vec<subjects::Model> = subjects::Entity::find()
            .apply_if(name, |query, name| {
                query.filter(subjects::Column::Name.eq(format!("%{name}%")))
            })
            .apply_if(instructor, |query, instructor| {
                query.filter(subjects::Column::InstructorId.eq(instructor))
            })
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .collect();

        let instructors: Vec<User> = subjects
            .load_one(users::Entity, self.as_ref())
            .await
            .into_iter()
            .flatten()
            .flatten()
            .map(User::from)
            .collect();

        Ok(itertools::izip!(subjects, instructors)
            .map(Subject::from)
            .collect())
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        subjects::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await?;
        Ok(())
    }
}

#[derive(Serialize)]
pub struct Subject {
    pub id: Uuid,
    pub name: String,
    pub instructor: User,
    pub cron_expr: String,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<(subjects::Model, User)> for Subject {
    fn from(
        (
            subjects::Model {
                id,
                name,
                cron_expr,
                create_at,
                updated_at,
                ..
            },
            instructor,
        ): (subjects::Model, User),
    ) -> Self {
        Self {
            id,
            name,
            instructor,
            cron_expr,
            create_at,
            updated_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateSubject {
    pub name: String,
    #[serde(skip)]
    pub instructor_id: Uuid,
    pub cron_expr: String,
}
