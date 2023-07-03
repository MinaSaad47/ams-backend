use std::sync::Arc;

use chrono::{Datelike, Utc};
use itertools::Itertools;
use sea_orm::{
    prelude::{async_trait::async_trait, *},
    QueryOrder, QueryTrait, Set, TransactionTrait,
};

use super::*;

use crate::{entity::subject_dates, prelude::*};

use crate::entity::{attendances, attendees, instructors, subjects};

pub struct AttendancesRepo(pub Arc<DatabaseConnection>);

impl AsRef<DatabaseConnection> for AttendancesRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[async_trait]
impl AttendancesRepoTrait for AttendancesRepo {
    async fn create_many(
        &self,
        subject_id: Uuid,
        attendee_ids: Vec<Uuid>,
    ) -> Result<Vec<Attendance>, RepoError> {
        let txn = self.as_ref().begin().await?;

        let mut created = Vec::with_capacity(attendee_ids.len());

        for attendee_id in attendee_ids {
            let attendance = self
                .create_one(CreateAttendance {
                    attendee_id,
                    subject_id,
                })
                .await?;
            created.push(attendance)
        }

        txn.commit().await?;

        Ok(created)
    }

    async fn create_one(&self, attendance: CreateAttendance) -> Result<Attendance, RepoError> {
        if let Some(last_attendance) = attendances::Entity::find()
            .order_by_desc(attendances::Column::CreateAt)
            .one(self.as_ref())
            .await?
        {
            if last_attendance.attendee_id == attendance.attendee_id {
                let now = Utc::now();
                let last = last_attendance.create_at.with_timezone(&now.timezone());

                if now.day() == last.day() {
                    return Err(RepoError::DuplicateAttendance {
                        attendee_id: attendance.attendee_id,
                        subject_id: attendance.subject_id,
                    });
                }
            }
        };

        let attendance: attendances::Model = attendances::ActiveModel {
            attendee_id: Set(attendance.attendee_id),
            subject_id: Set(attendance.subject_id),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await
        .map_duplicate(RepoError::DuplicateAttendance {
            attendee_id: attendance.attendee_id,
            subject_id: attendance.subject_id,
        })?;

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
            .await?
            .into_iter()
            .collect_vec();

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
            .collect_vec();

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

        let subjects: Vec<Subject> = itertools::izip!(subjects, dates, instructors)
            .map_into()
            .collect_vec();

        Ok(itertools::izip!(attendances, attendees, subjects)
            .map_into()
            .collect_vec())
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

        let dates = subject
            .find_related(subject_dates::Entity)
            .all(self.as_ref())
            .await?
            .into_iter()
            .map_into()
            .collect_vec();

        Ok((attendance, attendee, (subject, dates, instructor).into()).into())
    }
}
