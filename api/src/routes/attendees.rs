use std::env;

use axum::{
    extract::{Multipart, Path, State},
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{encode, Header};
use tokio::fs;
use uuid::Uuid;

use logic::prelude::*;
use nn_model::Embedding;

use crate::{
    app::{self, DynAttendancesRepo, DynAttendeesRepo, DynSubjectsRepo},
    auth::{AuthBody, AuthError, AuthPayload, Claims, User, KEYS},
    error::ApiError,
    response::{AppResponse, AppResponseDataExt, AppResponseMsgExt},
};

pub(crate) fn routes() -> Router<app::State> {
    Router::new()
        .route("/attendees", get(get_all).post(create_one))
        .route("/attendees/image", post(get_all_with_image))
        .route(
            "/attendees/:id",
            get(get_one).patch(update_one).delete(delete_one),
        )
        .route("/attendees/:id/image", post(upload_image))
        .route("/attendees/:id/subjects", get(get_all_subjects_for_one))
        .route(
            "/attendees/:id/subjects/:id",
            get(get_one_subject_for_one)
                .put(put_one_subject_to_one)
                .delete(delete_one_subject_from_one),
        )
        .route(
            "/attendees/:id/subjects/:id/attendances",
            get(get_all_attendances_with_one_attendee_and_one_subject),
        )
        .route(
            "/attendees/login",
            post(login_with_creds).get(login_with_token),
        )
}

/*
* Attendees Routes
*/

#[utoipa::path(
    get,
    path = "/attendees",
    responses(
        (status = OK, body = AttendeesListResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_all(
    State(repo): State<DynAttendeesRepo>,
    claimes: Claims,
) -> Result<AppResponse<'static, Vec<Attendee>>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let attendees = repo.get_all().await?;
    let response = attendees.ok_response("retreived all attendees successfully");

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/attendees/image",
    request_body(content = Image, content_type = "multipart/form-data"),
    responses(
        (status = OK, body = AttendeesListResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_all_with_image(
    State(repo): State<DynAttendeesRepo>,
    claimes: Claims,
    multipart: Option<Multipart>,
) -> Result<AppResponse<'static, Vec<Attendee>>, ApiError> {
    if let User::Attendee(_) = claimes.user {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let Some(mut multipart) = multipart else {
        return Err(ApiError::Internal);
    };

    let Some(field) = multipart.next_field().await.ok().flatten() else {
        return Err(ApiError::Internal);
    };

    let mut attendees = repo.get_all().await?;

    let image_path = env::temp_dir().join("image.png");

    fs::write(
        &image_path,
        field.bytes().await.map_err(|_| ApiError::Internal)?,
    )
    .await?;

    let embedding: Vec<f64> =
        Embedding::from_image(image_path.to_str().ok_or(ApiError::Internal)?).await?;

    let mut attendees_embeddings: Vec<(Attendee, f64)> = attendees
        .into_iter()
        .filter(|attendee| attendee.embedding.is_some())
        .map(|attendee| {
            let distance = attendee
                .embedding
                .as_ref()
                .expect("filtered attendees without embedding")
                .distance(&embedding);
            (attendee, distance)
        })
        .filter(|(_, distance)| distance < &0.6)
        .collect();

    attendees_embeddings.sort_by(|attendee1, attendee2| attendee1.1.total_cmp(&attendee2.1));

    attendees = attendees_embeddings
        .into_iter()
        .map(|(attendee, _)| attendee)
        .collect();

    let response = attendees.ok_response("retreived all attendees successfully");

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/attendees",
    request_body = CreateAttendee,
    responses(
        (status = CREATED, body = AttendeeResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn create_one(
    State(repo): State<DynAttendeesRepo>,
    claimes: Claims,
    Json(attendee): Json<CreateAttendee>,
) -> Result<AppResponse<'static, Attendee>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let attendee = repo.create(attendee).await?;
    let response = attendee.create_response("create on attendee successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/attendees/login",
    responses(
        (status = OK, body = AttendeeResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn login_with_token(
    State(repo): State<DynAttendeesRepo>,
    claimes: Claims,
) -> Result<AppResponse<'static, Attendee>, ApiError> {
    let User::Attendee(attendee_id) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };
    let attendee = repo.get_by_id(attendee_id).await?;
    let response = attendee.ok_response("logged in as attendee successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/attendees/{attendee_id}",
    responses(
        (status = OK, body = AttendeeResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_one(
    State(repo): State<DynAttendeesRepo>,
    Path(attendee_id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<'static, Attendee>, ApiError> {
    match claimes.user {
        User::Admin(_) => {}
        User::Attendee(id) if id == attendee_id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };
    let attendee = repo.get_by_id(attendee_id).await?;
    let response = attendee.ok_response("retreived an attendee successfully");

    Ok(response)
}

#[utoipa::path(
    patch,
    path = "/attendees/{attendee_id}",
    request_body = UpdateAttendee,
    responses(
        (status = OK, body = AttendeeResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn update_one(
    State(repo): State<DynAttendeesRepo>,
    Path(attendee_id): Path<Uuid>,
    claimes: Claims,
    Json(update_attendee): Json<UpdateAttendee>,
) -> Result<AppResponse<'static, Attendee>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let attendee = repo.update(attendee_id, update_attendee).await?;
    let response = attendee.ok_response("update the attendee successfully");

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/attendees/{attendee_id}",
    responses(
        (status = OK)
    ),
    security(("api_jwt_token" = []))
)]
async fn delete_one(
    State(repo): State<DynAttendeesRepo>,
    Path(attendee_id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<'static, ()>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.delete_by_id(attendee_id).await?;
    let response = "deleted one attendee successfully".response();

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/attendees/login",
    request_body = AuthPayload,
    responses(
        (status = OK, body = AuthResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn login_with_creds(
    State(repo): State<DynAttendeesRepo>,
    payload: Option<Json<AuthPayload>>,
) -> Result<AppResponse<'static, AuthBody>, ApiError> {
    let Some(Json(payload)) = payload else {
        return Err(AuthError::MissingCredentials.into());
    };

    let attendee = repo.get_by_email(payload.email).await?;

    if payload.password != attendee.password {
        return Err(AuthError::WrongCredentials.into());
    }

    let claims = Claims {
        exp: usize::max_value(),
        user: User::Attendee(attendee.id),
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

    let response = AuthBody { token }.ok_response("logged in as attendee successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/attendees/{attendee_id}/subjects",
    responses(
        (status = OK, body = SubjectsListResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_all_subjects_for_one(
    State(repo): State<DynSubjectsRepo>,
    Path(attendee_id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<'static, Vec<Subject>>, ApiError> {
    if let User::Attendee(id) = claimes.user {
        if id != attendee_id {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let subjects = repo
        .get(SubjectsFilter {
            attendee_id: Some(attendee_id),
            ..Default::default()
        })
        .await?;
    let response = subjects.ok_response("retreived associated subjects successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/attendees/{attendee_id}/subjects/{subject_id}",
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_one_subject_for_one(
    State(repo): State<DynSubjectsRepo>,
    Path((attendee_id, subject_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<'static, Subject>, ApiError> {
    match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id) if id == attendee_id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let subjects = repo
        .get(SubjectsFilter {
            id: Some(subject_id),
            attendee_id: Some(attendee_id),
            ..Default::default()
        })
        .await?;

    let Some(subject) = subjects.into_iter().next() else {
        return Err(RepoError::NotFound("subject".to_owned()).into());
    };

    let response = subject.ok_response("retreived associated subjects successfully");

    Ok(response)
}

#[utoipa::path(
    put,
    path = "/attendees/{attendee_id}/subjects/{subject_id}",
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn put_one_subject_to_one(
    State(repo): State<DynSubjectsRepo>,
    Path((attendee_id, subject_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<'static, ()>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.add_attendee(subject_id, attendee_id).await?;
    let response = "a subject was added to an attendee successfully".response();

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/attendees/{attendee_id}/subjects/{subject_id}",
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn delete_one_subject_from_one(
    State(repo): State<DynSubjectsRepo>,
    Path((attendee_id, subject_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<'static, ()>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.remove_attendee(subject_id, attendee_id).await?;
    let response = "a subject was removed from an attendee successfully".response();

    Ok(response)
}

// TODO: unauthorize other attendees
async fn get_all_attendances_with_one_attendee_and_one_subject(
    State(repo): State<DynAttendancesRepo>,
    Path((attendee_id, subject_id)): Path<(Uuid, Uuid)>,
    _: Claims,
) -> Result<AppResponse<'static, Vec<Attendance>>, ApiError> {
    let attendances = repo
        .get(AttendancesFilter {
            subject_id: Some(subject_id),
            attendee_id: Some(attendee_id),
        })
        .await?;

    let response = attendances.ok_response("retreived all subject attendances for an attendee");

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/attendees/{attendee_id}/image",
    request_body(content = Image, content_type = "multipart/form-data"),
    responses(
        (status = OK, body = AttendeesListResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn upload_image(
    State(repo): State<DynAttendeesRepo>,
    Path(attendee_id): Path<Uuid>,
    claimes: Claims,
    mut multipart: Multipart,
) -> Result<AppResponse<'static, Attendee>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let Ok(Some(field)) = multipart.next_field().await else {
        return Err(ApiError::Internal);
    };

    let image_path = env::temp_dir().join("image.png");

    let image = field
        .bytes()
        .await
        .map_err(|_| ApiError::Internal)?
        .to_vec();

    fs::write(&image_path, image.clone()).await?;

    let embedding = Vec::from_image(image_path.to_str().ok_or(ApiError::Internal)?).await?;

    let attendee = repo
        .update(
            attendee_id,
            UpdateAttendee {
                embedding: Some(Some(embedding)),
                image: Some(image),
                ..Default::default()
            },
        )
        .await?;

    let response = attendee.ok_response("added an image to an attendee successfully");

    Ok(response)
}

#[cfg(test)]
mod tests {
    use axum_test_helper::TestClient;
    use hyper::StatusCode;
    use rstest::{fixture, rstest};

    use super::*;

    #[fixture]
    fn client() -> TestClient {
        let db = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async move { crate::connect_to_database().await })
                .unwrap()
        });

        let state = app::State::new(db);

        let app = routes().with_state(state);

        TestClient::new(app)
    }

    #[rstest]
    #[case::valid_cred(AuthPayload::new("MinaAttedee@outlook.com", "12345678"))]
    #[should_panic]
    #[case::invalid_cred_password(AuthPayload::new("mina@saad.com", "saad"))]
    #[should_panic]
    #[case::invalid_cred_email(AuthPayload::new("mina", "474747"))]
    #[should_panic]
    #[case::invalid_cred_all(AuthPayload::new("mina", "saad"))]
    #[trace]
    #[tokio::test(flavor = "multi_thread")]
    async fn login_test(#[notrace] client: TestClient, #[case] auth_payload: AuthPayload) {
        let response = client
            .post("/attendees/login")
            .json(&auth_payload)
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let _ = response.json::<AppResponse<AuthBody>>().await;
    }

    #[rstest]
    #[should_panic]
    #[case::invalid_cred(AuthPayload::new("mina", "saad"))]
    #[case::valid_cred(AuthPayload::new("MinaAttedee@outlook.com", "12345678"))]
    #[trace]
    #[tokio::test(flavor = "multi_thread")]
    async fn get_all_for_self(#[notrace] client: TestClient, #[case] auth_payload: AuthPayload) {
        let response = client
            .post("/attendees/login")
            .json(&auth_payload)
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let AppResponse {
            data: Some(AuthBody { token }),
            ..
        } = response.json::<AppResponse<'static, AuthBody>>().await else {
            panic!();
        };

        let response = client
            .get("/attendees/")
            .header("Authorization", format!("Barear {token}"))
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let _: AppResponse<'static, Vec<Attendee>> = response.json().await;
    }
}
