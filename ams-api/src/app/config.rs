use std::env;

use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::sync::RwLock;

use tracing_subscriber::filter::LevelFilter;
use utoipa::ToSchema;

pub(crate) static DB_CONNECTION: Lazy<String> = Lazy::new(|| env::var("DATABASE_URL").unwrap());
pub(crate) static SECRET: Lazy<String> = Lazy::new(|| env::var("JWT_SECRET").unwrap());
pub(crate) const LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum FaceRecModeKind {
    Classify,
    Embed,
}

pub static FACE_REC_MODE: RwLock<FaceRecModeKind> = RwLock::const_new(FaceRecModeKind::Classify);
