
use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, DatabaseConnection, EntityTrait,
    LoaderTrait, ModelTrait, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{attendances, subjects, users},
    subjects::Subject,
    users::User,
};

#[async_trait]
pub trait AttendancesRepoTrait {
    async fn create(&self, attendance: CreateAttendance) -> Attendance;
    async fn delete_by_id(&self, id: Uuid);
    async fn get_all(&self) -> Vec<Attendance>;
    async fn get_by_id(&self, id: Uuid) -> Attendance;
}

pub struct AttendancesRepository(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for AttendancesRepository {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
impl AttendancesRepoTrait for AttendancesRepository {
    async fn create(&self, attendance: CreateAttendance) -> Attendance {
        let attendance: attendances::Model = attendances::ActiveModel {
            user_id: Set(attendance.user_id),
            subject_id: Set(attendance.subject_id),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await
        .unwrap()
        .into();

        self.get_by_id(attendance.id).await
    }
    async fn delete_by_id(&self, id: Uuid) {
        attendances::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await
            .unwrap();
    }
    async fn get_all(&self) -> Vec<Attendance> {
        let attendances: Vec<attendances::Model> = attendances::Entity::find()
            .all(self.as_ref())
            .await
            .into_iter()
            .flatten()
            .collect();

        let attendees = attendances
            .load_one(users::Entity, self.as_ref())
            .await
            .into_iter()
            .flatten()
            .flatten()
            .map(User::from);

        let subjects: Vec<subjects::Model> = attendances
            .load_one(subjects::Entity, self.as_ref())
            .await
            .into_iter()
            .flatten()
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

        let subjects: Vec<Subject> = itertools::izip!(subjects, instructors)
            .map(Subject::from)
            .collect();

        itertools::izip!(attendances, attendees, subjects)
            .map(Attendance::from)
            .collect()
    }
    async fn get_by_id(&self, id: Uuid) -> Attendance {
        let attendance = attendances::Entity::find_by_id(id)
            .one(self.as_ref())
            .await
            .unwrap()
            .unwrap();

        let attendee = attendance
            .find_related(users::Entity)
            .one(self.as_ref())
            .await
            .unwrap()
            .unwrap()
            .into();

        let subject = attendance
            .find_related(subjects::Entity)
            .one(self.as_ref())
            .await
            .unwrap()
            .unwrap();

        let instructor = subject
            .find_related(users::Entity)
            .one(self.as_ref())
            .await
            .unwrap()
            .unwrap()
            .into();

        (attendance, attendee, (subject, instructor).into()).into()
    }
}

#[derive(Serialize)]
pub struct Attendance {
    pub id: Uuid,
    pub attendee: User,
    pub subject: Subject,
    pub create_at: DateTime<FixedOffset>,
}

impl From<(attendances::Model, User, Subject)> for Attendance {
    fn from(
        (attendances::Model { id, create_at, .. }, user, subject): (
            attendances::Model,
            User,
            Subject,
        ),
    ) -> Self {
        Self {
            id,
            attendee: user,
            subject,
            create_at,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateAttendance {
    pub user_id: Uuid,
    pub subject_id: Uuid,
}
