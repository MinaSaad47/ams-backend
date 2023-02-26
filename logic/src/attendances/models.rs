use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::entity::attendances;
use crate::prelude::*;

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
