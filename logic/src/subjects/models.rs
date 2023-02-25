use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{entity::subjects, prelude::*};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
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
