use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, LoaderTrait, ModelTrait, QueryFilter, QueryTrait, Set,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    attendees::Attendee,
    database::{attendances, attendees, instructors, subjects},
    error::RepoError,
    instructors::Instructor,
    subjects::Subject,
};

#[async_trait]
pub trait AttendancesRepoTrait {
    async fn create(&self, attendance: CreateAttendance) -> Result<Attendance, RepoError>;
    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
    async fn get(&self, attendaces_filter: AttendancesFilter)
        -> Result<Vec<Attendance>, RepoError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Attendance, RepoError>;
}

pub struct AttendancesRepo(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for AttendancesRepo {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[cfg_attr(test, mockall::automock)]
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
            .ok_or(RepoError::NotFound("attendacne".to_owned()))?;

        let attendee = attendance
            .find_related(attendees::Entity)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("attendee".to_owned()))?
            .into();

        let subject = attendance
            .find_related(subjects::Entity)
            .one(self.as_ref())
            .await?
            .ok_or(RepoError::NotFound("subject".to_owned()))?;

        let instructor = subject
            .find_related(instructors::Entity)
            .one(self.as_ref())
            .await?
            .map(Instructor::from);

        Ok((attendance, attendee, (subject, instructor).into()).into())
    }
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Attendance {
    pub id: Uuid,
    pub attendee: Attendee,
    pub subject: Subject,
    pub create_at: DateTime<FixedOffset>,
}

impl From<(attendances::Model, Attendee, Subject)> for Attendance {
    fn from(
        (attendances::Model { id, create_at, .. }, attendee, subject): (
            attendances::Model,
            Attendee,
            Subject,
        ),
    ) -> Self {
        Self {
            id,
            attendee,
            subject,
            create_at,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttendance {
    pub attendee_id: Uuid,
    pub subject_id: Uuid,
}

#[derive(Default)]
pub struct AttendancesFilter {
    pub subject_id: Option<Uuid>,
    pub attendee_id: Option<Uuid>,
}
