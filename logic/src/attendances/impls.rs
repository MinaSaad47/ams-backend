use std::sync::Arc;

use sea_orm::{
    prelude::{async_trait::async_trait, *},
    QueryTrait, Set,
};

use super::*;

use crate::prelude::*;

use crate::entity::{attendances, attendees, instructors, subjects};

pub struct AttendancesRepo(pub Arc<DatabaseConnection>);

impl AsRef<DatabaseConnection> for AttendancesRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[async_trait]
impl AttendancesRepoTrait for AttendancesRepo {
    async fn create(&self, attendance: CreateAttendance) -> Result<Attendance, RepoError> {
        let attendance: attendances::Model = attendances::ActiveModel {
            attendee_id: Set(attendance.attendee_id),
            subject_id: Set(attendance.subject_id),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await?;

        Ok(self.get_by_id(attendance.id).await?)
    }
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError> {
        attendances::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await?;
        Ok(())
    }
    async fn get(&self, filter: AttendancesFilter) -> Result<Vec<Attendance>, RepoError> {
        let attendances: Vec<attendances::Model> = attendances::Entity::find()
            .apply_if(filter.subject_id, |query, subject| {
                query.filter(attendances::Column::SubjectId.eq(subject))
            })
            .apply_if(filter.attendee_id, |query, attendee| {
                query.filter(attendances::Column::SubjectId.eq(attendee))
            })
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .collect();

        let attendees = attendances
            .load_one(attendees::Entity, self.as_ref())
            .await
            .into_iter()
            .flatten()
            .flatten()
            .map(Attendee::from);

        let subjects: Vec<subjects::Model> = attendances
            .load_one(subjects::Entity, self.as_ref())
            .await
            .into_iter()
            .flatten()
            .flatten()
            .collect();

        let instructors: Vec<Option<Instructor>> = subjects
            .load_one(instructors::Entity, self.as_ref())
            .await?
            .into_iter()
            .map(|i| i.map(Instructor::from))
            .collect();

        let subjects: Vec<Subject> = itertools::izip!(subjects, instructors)
            .map(Subject::from)
            .collect();

        Ok(itertools::izip!(attendances, attendees, subjects)
            .map(Attendance::from)
            .collect())
    }
    async fn get_by_id(&self, id: Uuid) -> Result<Attendance, RepoError> {
        let attendance = attendances::Entity::find_by_id(id)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendacnes".to_owned()))?;

        let attendee = attendance
            .find_related(attendees::Entity)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendees".to_owned()))?
            .into();

        let subject = attendance
            .find_related(subjects::Entity)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("subjects".to_owned()))?;

        let instructor = subject
            .find_related(instructors::Entity)
            .one(self.as_ref())
            .await?
            .map(Instructor::from);

        Ok((attendance, attendee, (subject, instructor).into()).into())
    }
}
