use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::prelude::*;

use crate::entity::subject_dates;
use crate::entity::subjects;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubjectDate {
    pub id: Uuid,
    pub day_of_week: i32,
    #[schema(example = "21:56:04", value_type = String)]
    pub start_time: chrono::NaiveTime,
    #[schema(example = "22:56:04", value_type = String)]
    pub end_time: chrono::NaiveTime,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<subject_dates::Model> for SubjectDate {
    fn from(
        subject_dates::Model {
            id,
            day_of_week,
            start_time,
            end_time,
            create_at,
            updated_at,
            ..
        }: subject_dates::Model,
    ) -> Self {
        Self {
            id,
            day_of_week,
            start_time,
            end_time,
            create_at,
            updated_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub id: Uuid,
    pub name: String,
    pub instructor: Option<Instructor>,
    pub dates: Vec<SubjectDate>,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<(subjects::Model, Vec<SubjectDate>, Option<Instructor>)> for Subject {
    fn from(
        (
            subjects::Model {
                id,
                name,
                create_at,
                updated_at,
                ..
            },
            dates,
            instructor,
        ): (subjects::Model, Vec<SubjectDate>, Option<Instructor>),
    ) -> Self {
        Self {
            id,
            name,
            instructor,
            dates,
            create_at,
            updated_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubjectDate {
    pub day_of_week: i32,
    #[schema(example = "21:56:04", value_type = String)]
    pub start_time: chrono::NaiveTime,
    #[schema(example = "22:56:04", value_type = String)]
    pub end_time: chrono::NaiveTime,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubject {
    #[schema(example = "intro to computer science")]
    pub name: String,
}

#[derive(Deserialize, Serialize, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubject {
    #[schema(example = "updated intro to computer science")]
    pub name: Option<String>,
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
