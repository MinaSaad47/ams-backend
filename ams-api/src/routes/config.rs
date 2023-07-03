use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Query, State},
    routing::{post, put},
    Router,
};
use ams_facerec::FaceRecognizer;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{
    app::{
        self,
        config::{FaceRecModeKind, FACE_REC_MODE},
    },
    auth::{AuthError, Claims, User},
    error::ApiError,
    response::{AppResponse, AppResponseMsgExt},
};

pub(crate) fn routes() -> Router<app::State> {
    Router::new()
        .route("/config/face_recognition", put(face_recognition))
        .route("/config/classifier", post(upload_classifier))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct FaceRecognition {
    mode: FaceRecModeKind,
}

#[utoipa::path(
    put,
    path = "/config/face_recognition",
    params(FaceRecognition),
    responses(
        (status = OK)
    ),
    security(("api_jwt_token" = []))
)]
async fn face_recognition(
    Query(face_recogintion): Query<FaceRecognition>,
    claims: Claims,
) -> Result<AppResponse<'static, ()>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    *FACE_REC_MODE.write().await = face_recogintion.mode;

    let respone = "updated the face recognition mode successfulty".response();

    Ok(respone)
}

#[utoipa::path(
    post,
    path = "/config/classifier",
    request_body(content = Classifier, content_type = "multipart/form-data"),
    responses(
        (status = OK)
    ),
    security(("api_jwt_token" = []))
)]
async fn upload_classifier(
    State(face_recogition): State<Arc<FaceRecognizer>>,
    claims: Claims,
    mut multipart: Multipart,
) -> Result<AppResponse<'static, ()>, ApiError> {
    let User::Admin(_) = claims.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let Ok(Some(item)) = multipart.next_field().await else {
        return Err(ApiError::BadRequest);
    };

    let Some(_) = item.file_name() else {
        return Err(ApiError::BadRequest);
    };

    let classifier = item
        .bytes()
        .await
        .map_err(|_| ApiError::BadRequest)?
        .to_vec();

    face_recogition.upload_classifier(&classifier).await?;

    let respone = "updated the classifier successfulty".response();

    Ok(respone)
}
