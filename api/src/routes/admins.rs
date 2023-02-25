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
    use std::sync::Arc;

    use rstest::*;

    use axum::http::{Method, StatusCode};
    use logic::prelude::*;

    use mockall::predicate;

    use super::*;

    use crate::test_utils::*;

    #[fixture]
    fn testing_app() -> TestingApp {
        let dummy_admin = Admin {
            name: "Mina Saad".to_owned(),
            email: "mina@saad.com".to_owned(),
            password: "474747".to_owned(),
            ..Default::default()
        };
        let mut mocked_repo = MockAdminRepoStruct::new();
        mocked_repo
            .expect_get_by_email()
            .with(predicate::eq("mina@saad.com".to_owned()))
            .return_once(|_| Ok(dummy_admin));

        let admins_state = AdminsState {
            admins_repo: Arc::new(mocked_repo),
        };

        TestingApp::new(routes(admins_state), "/admins")
    }

    #[rstest]
    #[case::valid_cred("mina@saad.com", "474747")]
    #[should_panic]
    #[case::invalid_cred("invalid", "invalid")]
    #[tokio::test]
    #[trace]
    async fn login_test(
        #[notrace] mut testing_app: TestingApp,
        #[case] email: String,
        #[case] password: String,
    ) {
        let body = AuthPayload { email, password };

        let res: TestingResponse<AppResponse<AuthBody>> = testing_app
            .request("/login", Method::POST, Some(body))
            .await;

        assert_eq!(res.status, StatusCode::OK);

        let body = res.body.unwrap();

        assert!(body.data.is_some())
    }
}
