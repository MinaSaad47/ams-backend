use std::sync::Arc;

use itertools::Itertools;
use sea_orm::{
    prelude::{async_trait::async_trait, *},
    sea_query::Query,
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait, Set,
};
use uuid::Uuid;

pub use crate::prelude::*;

use crate::entity::{attendees, attendees_subjects, instructors, subject_dates, subjects};

pub struct SubjectsRepository(pub Arc<DatabaseConnection>);

impl AsRef<DatabaseConnection> for SubjectsRepository {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[async_trait]
impl SubjectsRepoTrait for SubjectsRepository {
    async fn create(&self, subject: CreateSubject) -> Result<Subject, RepoError> {
        let created_subject = subjects::ActiveModel {
            name: Set(subject.name),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await?;

        Ok((created_subject, vec![], None).into())
    }

    async fn remove_subject_date(
        &self,
        subject_id: Uuid,
        subject_date_id: Uuid,
    ) -> Result<(), RepoError> {
        let subject_date: subject_dates::ActiveModel = subject_dates::Entity::find()
            .filter(
                subject_dates::Column::Id
                    .eq(subject_date_id)
                    .and(subject_dates::Column::SubjectId.eq(subject_id)),
            )
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("subject_dates".to_owned()))?
            .into();

        subject_date.delete(self.as_ref()).await?;

        Ok(())
    }

    async fn add_subject_date(
        &self,
        subject_id: Uuid,
        CreateSubjectDate {
            day_of_week,
            start_time,
            end_time,
        }: CreateSubjectDate,
    ) -> Result<SubjectDate, RepoError> {
        let subject_date = subject_dates::ActiveModel {
            subject_id: Set(subject_id),
            day_of_week: Set(day_of_week),
            start_time: Set(start_time),
            end_time: Set(end_time),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await?;

        Ok(subject_date.into())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Subject, RepoError> {
        let subject = subjects::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("subjects".to_owned()))?;

        let instructor = subject
            .find_related(instructors::Entity)
            .one(self.as_ref())
            .await?
            .map(Instructor::from);

        let dates = subject
            .find_related(subject_dates::Entity)
            .all(self.as_ref())
            .await?
            .into_iter()
            .map_into()
            .collect_vec();

        Ok((subject, dates, instructor).into())
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
                qeury.filter(
                    subjects::Column::Id.in_subquery(
                        Query::select()
                            .column(attendees_subjects::Column::SubjectId)
                            .from(attendees_subjects::Entity)
                            .and_where(attendees_subjects::Column::AttendeeId.eq(attendee))
                            .to_owned(),
                    ),
                )
            })
            .all(self.as_ref())
            .await?
            .into_iter()
            .collect();

        let instructors: Vec<Option<Instructor>> = subjects
            .load_one(instructors::Entity, self.as_ref())
            .await?
            .into_iter()
            .map(|i| i.map(Instructor::from))
            .collect_vec();

        let dates = subjects
            .load_many(subject_dates::Entity, self.as_ref())
            .await?
            .into_iter()
            .map(|dates| dates.into_iter().map_into().collect_vec())
            .collect_vec();

        Ok(itertools::izip!(subjects, dates, instructors)
            .map_into()
            .collect_vec())
    }
    async fn update(
        &self,
        id: Uuid,
        UpdateSubject {
            name,
            instructor_id,
        }: UpdateSubject,
    ) -> Result<Subject, RepoError> {
        let mut subject: subjects::ActiveModel = subjects::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("subjects".to_owned()))?
            .into();

        if let Some(name) = name {
            subject.name = Set(name);
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

        let dates = subject
            .find_related(subject_dates::Entity)
            .all(self.as_ref())
            .await?
            .into_iter()
            .map_into()
            .collect_vec();

        Ok((subject, dates, instructor).into())
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
            .ok_or(RepoError::NotFound("attendees_subjects".to_owned()))?
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

    async fn get_all_attendees(&self, id: Uuid) -> Result<Vec<Attendee>, RepoError> {
        let attendees: Vec<attendees::Model> = attendees::Entity::find()
            .filter(
                attendees::Column::Id.in_subquery(
                    Query::select()
                        .column(attendees_subjects::Column::AttendeeId)
                        .from(attendees_subjects::Entity)
                        .and_where(attendees_subjects::Column::SubjectId.eq(id))
                        .to_owned(),
                ),
            )
            .all(self.as_ref())
            .await?;

        let attendees = attendees.into_iter().map_into().collect_vec();

        Ok(attendees)
    }
}
