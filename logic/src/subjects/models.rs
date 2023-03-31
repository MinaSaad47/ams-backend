use std::str::FromStr;

use chrono::{DateTime, FixedOffset};
use cron::Schedule;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::prelude::*;

use crate::entity::subjects;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    pub id: Uuid,
    pub name: String,
    pub instructor: Option<Instructor>,
    #[serde(
        serialize_with = "cron_serialize",
        deserialize_with = "cron_deserialize"
    )]
    pub cron_expr: Schedule,
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
            cron_expr: Schedule::from_str(&cron_expr).expect("valid expression from the database"),
            create_at,
            updated_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubject {
    #[schema(example = "intro to computer science")]
    pub name: String,
    #[schema(example = "* * * * * *", value_type = String)]
    #[serde(
        serialize_with = "cron_serialize",
        deserialize_with = "cron_deserialize"
    )]
    pub cron_expr: Schedule,
}

#[derive(Deserialize, Serialize, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSubject {
    #[schema(example = "updated intro to computer science")]
    pub name: Option<String>,
    #[schema(example = "* * * * * *", value_type = String)]
    #[serde(
        serialize_with = "opt_cron_serialize",
        deserialize_with = "opt_cron_deserialize"
    )]
    pub cron_expr: Option<Schedule>,
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

fn cron_serialize<S>(cron: &Schedule, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&cron.to_string())
}

fn opt_cron_serialize<S>(cron: &Option<Schedule>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match cron {
        Some(cron) => s.serialize_some(&cron.to_string()),
        None => s.serialize_none(),
    }
}

fn cron_deserialize<'de, D>(deserializer: D) -> Result<cron::Schedule, D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    cron::Schedule::from_str(&buf).map_err(serde::de::Error::custom)
}

fn opt_cron_deserialize<'de, D>(deserializer: D) -> Result<Option<cron::Schedule>, D::Error>
where
    D: Deserializer<'de>,
{
    let buf: Option<String> = Deserialize::deserialize(deserializer)?;

    let res = match buf {
        Some(buf) => Some(Schedule::from_str(&buf).map_err(serde::de::Error::custom)?),
        None => None,
    };

    Ok(res)
}
