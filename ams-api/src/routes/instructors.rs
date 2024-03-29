use axum::{
    extract::{Multipart, Path, State},
    routing::{get, post},
    Json, Router,
};
use jsonwebtoken::{encode, Header};
use uuid::Uuid;

use ams_logic::prelude::*;

use crate::{
    app::{self, DynInstructorsRepo, DynSubjectsRepo},
    auth::{AuthBody, AuthError, AuthPayload, Claims, User, KEYS},
    error::ApiError,
    response::{AppResponse, AppResponseDataExt, AppResponseMsgExt},
};

pub(crate) fn routes() -> Router<app::State> {
    Router::new()
        .route("/instructors", get(get_all).post(create_one))
        .route(
            "/instructors/:id",
            get(get_one).patch(update_one).delete(delete_one),
        )
        .route("/instructors/:id/subjects", get(get_all_subjects_for_one))
        .route("/instructors/:id/image", post(upload_image))
        .route(
            "/instructors/:id/subjects/:id",
            get(get_one_subject_for_one)
                .put(put_one_subject_to_one)
                .delete(delete_one_subject_from_one),
        )
        .route(
            "/instructors/login",
            post(login_with_creds).get(login_with_token),
        )
}

/*
* Instructors Routes
*/

#[utoipa::path(
    get,
    path = "/instructors",
    responses(
        (status = OK, body = InstructorsListResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_all(
    State(repo): State<DynInstructorsRepo>,
    claimes: Claims,
) -> Result<AppResponse<'static, Vec<Instructor>>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let instructors = repo.get_all().await?;
    let response = instructors.ok_response("retreived all instructors successfully");
    Ok(response)
}

#[utoipa::path(
    post,
    path = "/instructors",
    request_body = CreateInstructor,
    responses(
        (status = CREATED, body = InstructorResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn create_one(
    State(repo): State<DynInstructorsRepo>,
    claimes: Claims,
    Json(instructor): Json<CreateInstructor>,
) -> Result<AppResponse<'static, Instructor>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let instructor = repo.create(instructor).await?;
    let response = instructor.create_response("create on instructor successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/instructors/{instructor_id}",
    responses(
        (status = CREATED, body = InstructorResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_one(
    State(repo): State<DynInstructorsRepo>,
    Path(instructor_id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<'static, Instructor>, ApiError> {
    match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id) if id == instructor_id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let instructor = repo.get_by_id(instructor_id).await?;
    let response = instructor.ok_response("retreived an instructor successfully");

    Ok(response)
}

#[utoipa::path(
    patch,
    path = "/instructors/{instructor_id}",
    params(
        ("id" = Uuid, Path, description = "instructor id"),
    ),
    request_body = UpdateInstructor,
    responses(
        (status = OK, body = InstructorResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn update_one(
    State(repo): State<DynInstructorsRepo>,
    Path(instructor_id): Path<Uuid>,
    claimes: Claims,
    Json(update_instructor): Json<UpdateInstructor>,
) -> Result<AppResponse<'static, Instructor>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let instructor = repo.update(instructor_id, update_instructor).await?;
    let response = instructor.ok_response("update the instructor successfully");

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/instructors/{id}",
    responses(
        (status = OK)
    ),
    security(("api_jwt_token" = []))
)]
async fn delete_one(
    State(repo): State<DynInstructorsRepo>,
    Path(instructor_id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<'static, ()>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    repo.delete_by_id(instructor_id).await?;
    let response = "deleted one instructor successfully".response();

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/instructors/login",
    request_body = AuthPayload,
    responses(
        (status = OK, body = AuthResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn login_with_creds(
    State(repo): State<DynInstructorsRepo>,
    payload: Option<Json<AuthPayload>>,
) -> Result<AppResponse<'static, AuthBody>, ApiError> {
    let Some(Json(payload)) = payload else {
        return Err(AuthError::MissingCredentials.into());
    };

    let instructor = repo.get_by_email(payload.email).await?;

    if payload.password != instructor.password {
        return Err(AuthError::WrongCredentials.into());
    }

    let claims = Claims {
        exp: usize::max_value(),
        user: User::Instructor(instructor.id),
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

    let response = AuthBody { token }.ok_response("logged in as instructor successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/instructors/login",
    responses(
        (status = OK, body = AuthResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn login_with_token(
    State(repo): State<DynInstructorsRepo>,
    claimes: Claims,
) -> Result<AppResponse<'static, Instructor>, ApiError> {
    let User::Instructor(instructor_id) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    } ;

    let instructor = repo.get_by_id(instructor_id).await?;
    let response = instructor.ok_response("logged in as instructor successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/instructors/{instructor_id}/subjects",
    responses(
        (status = OK, body = SubjectsListResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_all_subjects_for_one(
    State(repo): State<DynSubjectsRepo>,
    Path(instructor_id): Path<Uuid>,
    claimes: Claims,
) -> Result<AppResponse<'static, Vec<Subject>>, ApiError> {
    match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id) if id == instructor_id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let subjects = repo
        .get(SubjectsFilter {
            instructor_id: Some(instructor_id),
            ..Default::default()
        })
        .await?;
    let response = subjects.ok_response("retreived associated subjects successfully");

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/instructors/{instructor_id}/subjects/{subject_id}",
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn get_one_subject_for_one(
    State(repo): State<DynSubjectsRepo>,
    Path((instructor_id, subject_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<'static, Subject>, ApiError> {
    match claimes.user {
        User::Admin(_) => {}
        User::Instructor(id) if id == instructor_id => {}
        _ => {
            return Err(AuthError::UnauthorizedAccess.into());
        }
    };

    let subjects = repo
        .get(SubjectsFilter {
            id: Some(subject_id),
            instructor_id: Some(instructor_id),
            ..Default::default()
        })
        .await?;

    let Some(subject) = subjects.into_iter().next() else {
        return Err(RepoError::NotFound("subject".to_owned()).into());
    };

    let response = subject.create_response("retreived associated subjects successfully");

    Ok(response)
}

#[utoipa::path(
    put,
    path = "/instructors/{instructor_id}/subjects/{subject_id}",
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn put_one_subject_to_one(
    State(repo): State<DynSubjectsRepo>,
    Path((instructor_id, subject_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<'static, Subject>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subject = repo
        .update(
            subject_id,
            UpdateSubject {
                instructor_id: Some(Some(instructor_id)),
                ..Default::default()
            },
        )
        .await?;

    let response = subject.create_response("assigned an instructor to a subject successfully");

    Ok(response)
}

#[utoipa::path(
    delete,
    path = "/instructors/{instructor_id}/subjects/{subject_id}",
    responses(
        (status = OK, body = SubjectResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn delete_one_subject_from_one(
    State(repo): State<DynSubjectsRepo>,
    Path((instructor_id, subject_id)): Path<(Uuid, Uuid)>,
    claimes: Claims,
) -> Result<AppResponse<'static, Subject>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let subject = repo.get_by_id(subject_id).await?;

    if let Some(instructor) = subject.instructor {
        if instructor.id != instructor_id {
            return Err(RepoError::NotFound(
                "no such instructor assigned for the subject".to_owned(),
            )
            .into());
        }
    }

    let subject = repo
        .update(
            subject_id,
            UpdateSubject {
                instructor_id: Some(None),
                ..Default::default()
            },
        )
        .await?;
    let response = subject.ok_response("removed the instructor from the subject successfully");

    Ok(response)
}

#[utoipa::path(
    post,
    path = "/instructors/{instructor_id}/image",
    request_body(content = Image, content_type = "multipart/form-data"),
    responses(
        (status = OK, body = InstructorResponse)
    ),
    security(("api_jwt_token" = []))
)]
async fn upload_image(
    State(repo): State<DynInstructorsRepo>,
    Path(instructor_id): Path<Uuid>,
    claimes: Claims,
    mut multipart: Multipart,
) -> Result<AppResponse<'static, Instructor>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let Ok(Some(field)) = multipart.next_field().await else {
        return Err(ApiError::Internal);
    };

    let image = field
        .bytes()
        .await
        .map_err(|_| ApiError::Internal)?
        .to_vec();

    let instructor = repo
        .update(
            instructor_id,
            UpdateInstructor {
                image: Some(image.into()),
                ..Default::default()
            },
        )
        .await?;

    let response = instructor.ok_response("added an image to an attendee successfully");

    Ok(response)
}
