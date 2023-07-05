mod app;
mod auth;
mod error;
mod openapi_docs;
mod response;
mod routes;
mod setup;

use std::net::SocketAddr;

use axum::{extract::DefaultBodyLimit, http::StatusCode, routing::get_service, Router};
use dotenvy::dotenv;
use openapi_docs::ApiDocs;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    normalize_path::NormalizePathLayer,
    services::ServeDir,
    trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::{admins, attendances, attendees, config, instructors, subjects};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load envirnment variables from .env
    dotenv().ok();

    // enable tracing
    setup::tracing();

    // connect to the database
    let db = setup::database_connection().await?;
    tracing::info!("connected the database successfully");

    // construct app state
    let state = app::State::new(db, app::config::ASSETS_DIR.as_str());

    let assets =
        get_service(ServeDir::new("assets")).handle_error(|error: std::io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        });

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDocs::openapi()))
        .nest(
            "/api",
            Router::new()
                .merge(config::routes())
                .merge(admins::routes())
                .merge(instructors::routes())
                .merge(attendances::routes())
                .merge(attendees::routes())
                .merge(subjects::routes())
                .with_state(state),
        )
        .nest_service("/assets", assets)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                        .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                        .on_failure(DefaultOnFailure::new().level(tracing::Level::INFO)),
                )
                .layer(NormalizePathLayer::trim_trailing_slash())
                .layer(CompressionLayer::new())
                .layer(DefaultBodyLimit::max(1024 * 1024 * 50)),
        );

    // start serving
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
