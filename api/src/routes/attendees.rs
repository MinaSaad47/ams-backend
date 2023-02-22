use std::env;

use axum::{
    extract::{FromRef, Multipart, Path, State},
    routing::{get, post},
    Json, Router,
};

use jsonwebtoken::{encode, Header};
use logic::{
    attendances::{Attendance, AttendancesFilter},
    attendees::{Attendee, CreateAttendee, UpdateAttendee},
    error::RepoError,
    subjects::{Subject, SubjectsFilter},
};
use nn_model::Embbedding;
use tokio::fs;
use uuid::Uuid;

use crate::{
    auth::{AuthBody, AuthError, AuthPayload, Claims, User, KEYS},
    error::ApiError,
    response::AppResponse,
    DynAttendancesRepo, DynAttendeesRepo, DynSubjectsRepo,
};

#[derive(Clone, FromRef)]
pub struct AttendeesState {
    pub attendees_repo: DynAttendeesRepo,
    pub subjects_repo: DynSubjectsRepo,
    pub attedances_repo: DynAttendancesRepo,
}

pub fn routes(attendees_state: AttendeesState) -> Router {
    Router::new()
        .route("/attendees", get(get_all).post(create_one))
        .route(
            "/attendees/:id",
            get(get_one).patch(update_one).delete(delete_one),
        )
        .route("/attendees/:id/image", post(upload_image))
        .route("/attendees/<id>/subjects", post(get_all_subjects_for_one))
        .route(
            "/attendees/<id>/subjects/<id>",
            get(get_one_subject_for_one)
                .put(put_one_subject_to_one)
                .delete(delete_one_subject_from_one),
        )
        .route(
            "/attendees/<id>/subjects/<id>/attendances",
            get(get_all_attendances_with_one_attendee_and_one_subject),
        )
        .route("/attendees/login", post(login))
        .with_state(attendees_state)
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
    multipart: Option<Multipart>,
) -> Result<AppResponse<Vec<Attendee>>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let mut attendees = repo.get_all().await?;

    if let Some(mut multipart) = multipart {
        let Ok(Some(field)) = multipart.next_field().await  else {
            return Err(ApiError::Unknown);
        } ;

        let image_path = env::temp_dir().join("image.png");

        fs::write(&image_path, field.bytes().await.unwrap())
            .await
            .unwrap();

        let embedding: Vec<f64> = Embbedding::from_image(image_path.to_str().unwrap())
            .await
            .unwrap();

        let mut attendees_embeddings: Vec<(Attendee, f64)> = attendees
            .into_iter()
            .filter(|attendee| attendee.embedding.is_some())
            .map(|attendee| {
                let distance = attendee.embedding.as_ref().unwrap().distance(&embedding);
                (attendee, distance)
            })
            .filter(|(_, distance)| distance < &0.6)
            .collect();

        attendees_embeddings.sort_by(|attendee1, attendee2| attendee1.1.total_cmp(&attendee2.1));

        attendees = attendees_embeddings
            .into_iter()
            .map(|(attendee, _)| attendee)
            .collect();
    };

    let response = AppResponse::created(attendees, "retreived all attendees successfully");

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
) -> Result<AppResponse<Attendee>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let attendee = repo.create(attendee).await?;
    let response = AppResponse::created(attendee, "create on attendee successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/attendees/{id}",
    params(
        ("id" = Uuid, Path, description = "instructor id"),
    ),
    responses(
        (status = CREATED, body = AttendeeResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_one(
    State(repo): State<DynAttendeesRepo>,
    Path(id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<Attendee>, ApiError> {
    let _ = match claimes.user {
        User::Admin(_) => {}
        User::Attendee(id) if id == id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let attendee = repo.get_by_id(id).await?;
    let response = AppResponse::with_content(attendee, "retreived an attendee successfully");

    Ok(response)
}

#[utoipa::path(
    patch,
    path = "/attendees/{id}",
    params(
        ("id" = Uuid, Path, description = "instructor id"),
    ),
    request_body = UpdateAttendee,
    responses(
        (status = OK, body = AttendeeResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn update_one(
    State(repo): State<DynAttendeesRepo>,
    Path(id): Path<Uuid>,
    claimes: Claims,
    Json(update_attendee): Json<UpdateAttendee>,
) -> Result<AppResponse<Attendee>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let attendee = repo.update(id, update_attendee).await?;
    let response = AppResponse::with_content(attendee, "update the attendee successfully");

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/attendees/{id}",
    params(
        ("id" = Uuid, Path, description = "instructor id"),
    ),
    responses(
        (status = OK)
    ),
    security(("api_jwt_token" = []))
)]
async fn delete_one(
    State(repo): State<DynAttendeesRepo>,
    Path(id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<()>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.delete_by_id(id).await?;
    let response = AppResponse::no_content("deleted one instructor successfully");

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/attendees/login",
    request_body = AuthPayload,
    responses(
        (status = OK, body = AuthResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn login(
    State(repo): State<DynAttendeesRepo>,
    Json(payload): Json<AuthPayload>,
) -> Result<AppResponse<AuthBody>, ApiError> {
    let attendee = repo.get_by_email(payload.email).await?;

    if payload.password != attendee.password {
        return Err(AuthError::WrongCredentials.into());
    }

    let claims = Claims {
        exp: usize::max_value(),
        user: User::Attendee(attendee.id),
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

    let response =
        AppResponse::with_content(AuthBody { token }, "logged in as attendee successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/attendees/{attendee_id}/subjects",
    params(
        ("attendee_id" = Uuid, Path, description = "instructor id"),
    ),
    responses(
        (status = OK, body = SubjectsListResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_all_subjects_for_one(
    State(repo): State<DynSubjectsRepo>,
    Path(attendee_id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<Vec<Subject>>, ApiError> {
    let _ = match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id) if id == attendee_id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let subjects = repo
        .get(SubjectsFilter {
            attendee_id: Some(attendee_id),
            ..Default::default()
        })
        .await?;
    let response =
        AppResponse::with_content(subjects, "retreived associated subjects successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/attendees/{attendee_id}/subjects/{subject_id}",
    params(
        ("attendee_id" = Uuid, Path, description = "instructor id"),
        ("subject_id" = Uuid, Path, description = "subject id"),
    ),
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_one_subject_for_one(
    State(repo): State<DynSubjectsRepo>,
    Path((attendee_id, subject_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<Subject>, ApiError> {
    let _ = match claimes.user {
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

    let Some(subject) = subjects.into_iter().nth(0) else {
        return Err(RepoError::NotFound("subject".to_owned()).into());
    };

    let response = AppResponse::with_content(subject, "retreived associated subjects successfully");

    Ok(response)
}

#[utoipa::path(
    put,
    path = "/attendees/{attendee_id}/subjects/{subject_id}",
    params(
        ("attendee_id" = Uuid, Path, description = "instructor id"),
        ("subject_id" = Uuid, Path, description = "subject id"),
    ),
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn put_one_subject_to_one(
    State(repo): State<DynSubjectsRepo>,
    Path((attendee_id, subject_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<()>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.add_attendee(subject_id, attendee_id).await?;
    let response = AppResponse::no_content("a subject was added to an attendee successfully");

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/attendees/{attendee_id}/subjects/{subject_id}",
    params(
        ("attendee_id" = Uuid, Path, description = "instructor id"),
        ("subject_id" = Uuid, Path, description = "subject id"),
    ),
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn delete_one_subject_from_one(
    State(repo): State<DynSubjectsRepo>,
    Path((attendee_id, subject_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<()>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.remove_attendee(subject_id, attendee_id).await?;
    let response = AppResponse::no_content("a subject was removed from an attendee successfully");

    Ok(response)
}

async fn get_all_attendances_with_one_attendee_and_one_subject(
    State(repo): State<DynAttendancesRepo>,
    Path((attendee_id, subject_id)): Path<(Uuid, Uuid)>,
    _: Claims,
) -> Result<AppResponse<Vec<Attendance>>, ApiError> {
    let attendances = repo
        .get(AttendancesFilter {
            subject_id: Some(subject_id),
            attendee_id: Some(attendee_id),
        })
        .await?;

    let response = AppResponse::with_content(
        attendances,
        "retreived all subject attendances for an attendee",
    );

    Ok(response)
}

async fn upload_image(
    State(repo): State<DynAttendeesRepo>,
    Path(id): Path<Uuid>,
    claimes: Claims,
    mut multipart: Multipart,
) -> Result<AppResponse<()>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let Ok(Some(field)) = multipart.next_field().await else {
        return Err(ApiError::Unknown.into());
    };

    let image_path = env::temp_dir().join("image.png");

    fs::write(&image_path, field.bytes().await.unwrap())
        .await
        .unwrap();

    let embedding = Vec::from_image(image_path.to_str().unwrap()).await.unwrap();

    repo.update(
        id,
        UpdateAttendee {
            embedding: Some(Some(embedding)),
            ..Default::default()
        },
    )
    .await?;

    let response = AppResponse::no_content("added an image to an attendee successfully");

    Ok(response)
}
