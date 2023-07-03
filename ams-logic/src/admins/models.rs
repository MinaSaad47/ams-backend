use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::entity::admins;

#[derive(Deserialize, Serialize, Debug, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct Admin {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub create_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<admins::Model> for Admin {
    fn from(
        admins::Model {
            id,
            name,
            email,
            password,
            create_at,
            updated_at,
        }: admins::Model,
    ) -> Self {
        Self {
            id,
            name,
            email,
            password,
            create_at,
            updated_at,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAdmin {
    pub name: String,
    pub email: String,
    pub password: String,
}
