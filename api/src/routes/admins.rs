use axum::{
    extract::{FromRef, State},
    routing::post,
    Json, Router,
};
use jsonwebtoken::{encode, Header};

use crate::{
    auth::{AuthBody, AuthError, AuthPayload, Claims, User, KEYS},
    error::ApiError,
    response::AppResponse,
    DynAdminsRepo,
};

#[derive(Clone, FromRef)]
pub struct AdminsState {
    pub admins_repo: DynAdminsRepo,
}

pub fn routes(admins_state: AdminsState) -> Router {
    Router::new()
        .route("/admins/login", post(login))
        .with_state(admins_state)
}

#[utoipa::path(
    post,
    path = "/admins/login",
    request_body = AuthPayload,
    responses(
        (status = OK, body = AuthResponse)
    ),
)]
async fn login(
    State(repo): State<DynAdminsRepo>,
    Json(payload): Json<AuthPayload>,
) -> Result<AppResponse<AuthBody>, ApiError> {
    let admin = repo.get_by_email(payload.email).await?;

    if payload.password != admin.password {
        return Err(AuthError::WrongCredentials.into());
    }

    let claims = Claims {
        exp: usize::max_value(),
        user: User::Admin(admin.id),
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

    let response = AppResponse::with_content(AuthBody { token }, "logged in as admin successfully");

    Ok(response)
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use rstest::*;

    use axum::http::{Method, StatusCode};
    use logic::{get_testing_db, prelude::*};

    use crate::test_utils::*;

    use super::*;

    #[fixture]
    #[once]
    fn testing_app() -> Mutex<TestingApp> {
        let db = get_testing_db(dotenvy_macro::dotenv!("DATABASE_BASE_URL"), "admin_testing");

        let admins_repo = Arc::new(AdminsRepoPg(Arc::new(db)));

        let admins_state = AdminsState { admins_repo };

        Mutex::new(TestingApp::new(routes(admins_state), "/admins"))
    }

    #[rstest]
    #[case::valid_cred("mina@saad.com", "474747")]
    #[should_panic]
    #[case::invalid_cred_password("mina@saad.com", "invalid")]
    #[should_panic]
    #[case::invalid_cred_email("invalid", "474747")]
    #[should_panic]
    #[case::invalid_cred_all("invalid", "invalid")]
    #[tokio::test(flavor = "multi_thread")]
    #[trace]
    async fn login_test(
        #[notrace] testing_app: &Mutex<TestingApp>,
        #[case] email: String,
        #[case] password: String,
    ) {
        let body = AuthPayload { email, password };

        let res: TestingResponse<AppResponse<AuthBody>> = testing_app
            .lock()
            .unwrap()
            .request("/login", Method::POST, Some(body))
            .await;

        dbg!(&res);

        assert_eq!(res.status, StatusCode::OK);

        let body = res.body.unwrap();

        assert!(body.data.is_some())
    }
}
