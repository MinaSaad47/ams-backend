use tracing::log;
use tracing_subscriber::{filter, prelude::*};

/// setup tracing for monitoring events.
pub(crate) fn tracing() {
    let layer1 = tracing_subscriber::fmt::layer()
        .pretty()
        .with_test_writer()
        .with_filter(filter::filter_fn(|meta| {
            meta.target().contains("api") || meta.target().contains("logic")
        }))
        .with_filter(config::LOG_LEVEL);

    let layer2 = tracing_subscriber::fmt::layer()
        .pretty()
        .with_test_writer()
        .with_line_number(false)
        .with_file(false)
        .with_thread_names(false)
        .with_target(false)
        .with_filter(filter::filter_fn(|meta| {
            meta.target().contains("sea_orm")
                || meta.target().contains("sqlx")
                || meta.target().contains("tower_http")
        }))
        .with_filter(config::LOG_LEVEL);

    tracing_subscriber::registry()
        .with(layer1)
        .with(layer2)
        .init();
}

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use crate::app::config;

/// connects to postgres database.
///
/// # Errors
///
/// This function will return an error if the connection can't be established.
#[tracing::instrument(err)]
pub(crate) async fn database_connection() -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(config::DB_CONNECTION.to_owned());
    opt.sqlx_logging_level(log::LevelFilter::Debug);

    tracing::info!("establishing database connection");
    tracing::debug!("connecting with options: \n{opt:#?}");

    Database::connect(opt).await
}
