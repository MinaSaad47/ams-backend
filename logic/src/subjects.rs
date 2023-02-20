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
    async fn get(&self, filter: SubjectsFilter) -> Result<Vec<Subject>, RepoError>;
    async fn update(&self, id: Uuid, update_subject: UpdateSubject) -> Result<Subject, RepoError>;
    async fn add_attendee(&self, id: Uuid, attendee_id: Uuid) -> Result<(), RepoError>;
    async fn remove_attendee(&self, id: Uuid, attendee_id: Uuid) -> Result<(), RepoError>;
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
    async fn get(&self, filter: SubjectsFilter) -> Result<Vec<Subject>, RepoError> {
        let subjects: Vec<subjects::Model> = subjects::Entity::find()
            .apply_if(filter.id, |query, id| {
                query.filter(subjects::Column::Id.eq(id))
            })
            .apply_if(filter.name, |query, name| {
                query.filter(subjects::Column::Name.eq(format!("%{name}%")))
            })
            .apply_if(filter.instructor_id, |query, instructor| {
                query.filter(subjects::Column::InstructorId.eq(instructor))
            })
            .apply_if(filter.attendee_id, |qeury, attendee| {
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
        UpdateSubject {
            name,
            cron_expr,
            instructor_id,
        }: UpdateSubject,
    ) -> Result<Subject, RepoError> {
        let mut subject: subjects::ActiveModel = subjects::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("subject".to_owned()))?
            .into();

        if let Some(name) = name {
            subject.name = Set(name);
        }
        if let Some(cron_expr) = cron_expr {
            subject.cron_expr = Set(cron_expr);
        }
        if let Some(instructor_id) = instructor_id {
            subject.instructor_id = Set(instructor_id);
        }

        let subject: subjects::Model = subject.update(self.as_ref()).await?;

        let instructor = subject
            .find_related(instructors::Entity)
            .one(self.as_ref())
            .await?
            .map(Instructor::from);

        Ok((subject, instructor).into())
    }
    async fn add_attendee(&self, id: Uuid, attendee_id: Uuid) -> Result<(), RepoError> {
        attendees_subjects::ActiveModel {
            subject_id: Set(id),
            attendee_id: Set(attendee_id),
        }
        .insert(self.as_ref())
        .await?;
        Ok(())
    }
    async fn remove_attendee(&self, id: Uuid, attendee_id: Uuid) -> Result<(), RepoError> {
        let attedee_subject: attendees_subjects::ActiveModel = attendees_subjects::Entity::find()
            .filter(
                attendees_subjects::Column::AttendeeId
                    .eq(attendee_id)
                    .and(attendees_subjects::Column::SubjectId.eq(id)),
            )
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound(
                "subject don't belog to attendee".to_owned(),
            ))?
            .into();
        attedee_subject.delete(self.as_ref()).await?;
        Ok(())
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

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubject {
    pub name: Option<String>,
    pub cron_expr: Option<String>,
    #[serde(skip)]
    pub instructor_id: Option<Option<Uuid>>,
}

#[derive(Default)]
pub struct SubjectsFilter {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub instructor_id: Option<Uuid>,
    pub attendee_id: Option<Uuid>,
}
