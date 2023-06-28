use serde::Deserialize;
use tokio::sync::RwLock;

use dotenvy_macro::dotenv;
use tracing_subscriber::filter::LevelFilter;

pub(crate) const DB_CONNECTION: &str = dotenv!("DATABASE_URL");
pub(crate) const SECRET: &str = dotenv!("JWT_SECRET");
pub(crate) const LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

#[derive(Deserialize)]
pub enum FaceRecModeKind {
    Classify,
    Embed,
}

pub static FACE_REC_MODE: RwLock<FaceRecModeKind> = RwLock::const_new(FaceRecModeKind::Classify);
