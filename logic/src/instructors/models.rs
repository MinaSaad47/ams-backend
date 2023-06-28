use std::borrow::Cow;

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use super::*;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Instructor {
    pub id: Uuid,
    pub number: i64,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub image: Option<String>,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<instructors::Model> for Instructor {
    fn from(
        instructors::Model {
            id,
            number,
            name,
            email,
            password,
            image,
            create_at,
            updated_at,
        }: instructors::Model,
    ) -> Self {
        Self {
            id,
            name,
            number,
            image,
            email,
            password,
            create_at,
            updated_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateInstructor {
    #[schema(example = "Mina Instructor")]
    pub name: String,
    #[schema(example = "MinaInstructor@outlook.com")]
    pub email: String,
    #[schema(example = "12345678")]
    pub password: String,
    #[schema(example = 13213321)]
    pub number: i64,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstructor{
    #[schema(example = "Emil Instructor")]
    pub name: Option<String>,
    #[schema(example = "EmilInstructor@outlook.com")]
    pub email: Option<String>,
    #[schema(example = "12345678")]
    pub password: Option<String>,
    #[serde(skip)]
    pub image: Option<Cow<'static, [u8]>>,
    #[schema(example = 3232323)]
    pub number: Option<i64>,
}
