use axum::{
    extract::{FromRef, Path, State},
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
        .route(
            "/attendees",
            get(get_all_attendees).post(create_one_attendee),
        )
        .route(
            "/attendees/:id",
            get(get_one_attendee)
                .patch(update_one_attendee)
                .delete(delete_one_attendee),
        )
        .route(
            "/attendees/<id>/subjects",
            post(get_all_subjects_for_one_attendee),
        )
        .route(
            "/attendees/<id>/subjects/<id>",
            get(get_one_subject_for_one_attendee)
                .put(put_one_subject_to_one_attendee)
                .delete(delete_one_subject_from_one_attendee),
        )
        .route("/attendees/<id>/subjects/<id>", post(login_one_attendee))
        .route(
            "/attendees/<id>/subjects/<id>/attendances",
            get(get_all_attendances_with_one_attendee_and_one_subject),
        )
        .route("/attendees/login", post(login_one_attendee))
        .with_state(attendees_state)
}

/*
* Attendees Routes
*/

async fn get_all_attendees(
    State(repo): State<DynAttendeesRepo>,
    claimes: Claims,
) -> Result<AppResponse<Vec<Attendee>>, ApiError> {
    let User::Admin(_) = claimes.user else {
        return Err(AuthError::UnauthorizedAccess.into());
    };

    let attendees = repo.get_all().await?;
    let response = AppResponse::created(attendees, "retreived all attendees successfully");

    Ok(response)
}

async fn create_one_attendee(
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

async fn get_one_attendee(
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

async fn update_one_attendee(
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

async fn delete_one_attendee(
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

async fn login_one_attendee(
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

async fn get_all_subjects_for_one_attendee(
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

async fn get_one_subject_for_one_attendee(
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

async fn put_one_subject_to_one_attendee(
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

async fn delete_one_subject_from_one_attendee(
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
