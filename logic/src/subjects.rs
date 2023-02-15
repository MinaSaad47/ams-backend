
use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::async_trait::async_trait, ActiveModelTrait, DatabaseConnection, EntityTrait,
    LoaderTrait, ModelTrait, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::{subjects, users},
    users::User,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SubjectsRepoTrait {
    async fn create(&self, subject: CreateSubject) -> Subject;
    async fn get_by_id(&self, id: Uuid) -> Subject;
    async fn get_all(&self) -> Vec<Subject>;
    async fn delete_by_id(&self, id: Uuid);
}

pub struct SubjectsRepository(pub DatabaseConnection);

impl AsRef<DatabaseConnection> for SubjectsRepository {
    fn as_ref(&self) -> &DatabaseConnection {
        &self.0
    }
}

#[async_trait]
impl SubjectsRepoTrait for SubjectsRepository {
    async fn create(&self, subject: CreateSubject) -> Subject {
        let subject = subjects::ActiveModel {
            name: Set(subject.name),
            instructor_id: Set(subject.instructor_id),
            cron_expr: Set(subject.cron_expr),
            ..Default::default()
        }
        .insert(self.as_ref())
        .await
        .unwrap();

        self.get_by_id(subject.id).await
    }
    async fn get_by_id(&self, id: Uuid) -> Subject {
        let subject = subjects::Entity::find_by_id(id)
            .one(self.as_ref())
            .await
            .unwrap()
            .unwrap();

        let instructor = subject
            .find_related(users::Entity)
            .one(self.as_ref())
            .await
            .unwrap()
            .unwrap();

        (subject, instructor.into()).into()
    }
    async fn get_all(&self) -> Vec<Subject> {
        let subjects: Vec<subjects::Model> = subjects::Entity::find()
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

        itertools::izip!(subjects, instructors)
            .map(Subject::from)
            .collect()
    }
    async fn delete_by_id(&self, id: Uuid) {
        subjects::Entity::delete_by_id(id)
            .exec(self.as_ref())
            .await
            .unwrap();
    }
}

#[derive(Serialize)]
pub struct Subject {
    id: Uuid,
    name: String,
    instructor: User,
    cron_expr: String,
    create_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
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
    pub instructor_id: Uuid,
    pub cron_expr: String,
}
