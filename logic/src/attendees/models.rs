use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::entity::attendees;

#[derive(Deserialize, Serialize, Debug, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct Attendee {
    pub id: Uuid,
    pub number: i64,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    #[serde(skip)]
    pub embedding: Option<Vec<f64>>,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<attendees::Model> for Attendee {
    fn from(
        attendees::Model {
            id,
            number,
            name,
            email,
            password,
            create_at,
            updated_at,
            embedding,
        }: attendees::Model,
    ) -> Self {
        Self {
            id,
            name,
            number,
            email,
            password,
            create_at,
            updated_at,
            embedding,
        }
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttendee {
    #[schema(example = "Mina Attedee")]
    pub name: String,
    #[schema(example = "MinaAttedee@outlook.com")]
    pub email: String,
    #[schema(example = "12345678")]
    pub password: String,
    #[schema(example = 13213321)]
    pub number: i64,
}
#[derive(Deserialize, Debug, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAttendee {
    #[schema(example = "Emil Attedee")]
    pub name: Option<String>,
    #[schema(example = "EmilAttedee@outlook.com")]
    pub email: Option<String>,
    #[schema(example = "12345678")]
    pub password: Option<String>,
    #[schema(example = 3232323)]
    pub number: Option<i64>,
    #[serde(skip)]
    pub embedding: Option<Option<Vec<f64>>>,
}
