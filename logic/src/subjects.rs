use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, JoinType, LoaderTrait, ModelTrait, QueryFilter, QuerySelect, QueryTrait,
    RelationTrait, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{attendees, attendees_subjects, instructors, subjects},
    error::RepoError,
    instructors::Instructor,
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
        attendee: Option<Uuid>,
    ) -> Result<Vec<Subject>, RepoError>;
    async fn update(&self, id: Uuid, update_subject: UpdateSubject) -> Result<Subject, RepoError>;
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
            .find_related(instructors::Entity)
            .one(self.as_ref())
            .await?
            .map(Instructor::from);

        Ok((subject, instructor).into())
    }
    async fn get(
        &self,
        name: Option<String>,
        instructor: Option<Uuid>,
        attendee: Option<Uuid>,
    ) -> Result<Vec<Subject>, RepoError> {
        let subjects: Vec<subjects::Model> = subjects::Entity::find()
            .apply_if(name, |query, name| {
                query.filter(subjects::Column::Name.eq(format!("%{name}%")))
            })
            .apply_if(instructor, |query, instructor| {
                query.filter(subjects::Column::InstructorId.eq(instructor))
            })
            .apply_if(attendee, |qeury, attendee| {
                qeury
                    .join(
                        JoinType::LeftJoin,
                        attendees_subjects::Relation::Attendees.def(),
                    )
                    .filter(attendees::Column::Id.eq(attendee))
            })
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .collect();

        let instructors: Vec<Option<Instructor>> = subjects
            .load_one(instructors::Entity, self.as_ref())
            .await?
            .into_iter()
            .map(|i| i.map(Instructor::from))
            .collect();

        Ok(itertools::izip!(subjects, instructors)
            .map(Subject::from)
            .collect())
    }
    async fn update(
        &self,
        id: Uuid,
        UpdateSubject { name, cron_expr }: UpdateSubject,
    ) -> Result<Subject, RepoError> {
        let mut subject: subjects::ActiveModel = subjects::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendee".to_owned()))?
            .into();

        if let Some(name) = name {
            subject.name = Set(name);
        }
        if let Some(cron_expr) = cron_expr {
            subject.cron_expr = Set(cron_expr);
        }

        let subject: subjects::Model = subject.update(self.as_ref()).await?;

        let instructor = subject
            .find_related(instructors::Entity)
            .one(self.as_ref())
            .await?
            .map(Instructor::from);

        Ok((subject, instructor).into())
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
    pub instructor: Option<Instructor>,
    pub cron_expr: String,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<(subjects::Model, Option<Instructor>)> for Subject {
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
        ): (subjects::Model, Option<Instructor>),
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
#[serde(rename_all = "camelCase")]
pub struct CreateSubject {
    pub name: String,
    pub cron_expr: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubject {
    pub name: Option<String>,
    pub cron_expr: Option<String>,
}
