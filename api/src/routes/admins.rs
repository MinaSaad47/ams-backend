use axum::{extract::State, routing::post, Json, Router};
use jsonwebtoken::{encode, Header};

use crate::{
    app::{self, DynAdminsRepo},
    auth::{AuthBody, AuthError, AuthPayload, Claims, User, KEYS},
    error::ApiError,
    response::AppResponse,
    response::AppResponseDataExt,
};

pub(crate) fn routes() -> Router<app::State> {
    Router::new().route("/admins/login", post(login))
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
    payload: Option<Json<AuthPayload>>,
) -> Result<AppResponse<'static, AuthBody>, ApiError> {
    let Some(Json(payload)) = payload else {
        return Err(AuthError::MissingCredentials.into());
    };

    let admin = repo.get_by_email(payload.email).await?;

    if payload.password != admin.password {
        return Err(AuthError::WrongCredentials.into());
    }

    let claims = Claims {
        exp: usize::max_value(),
        user: User::Admin(admin.id),
    };

    let token = encode(&Header::default(), &claims, &KEYS.encoding).unwrap();

    let response = AuthBody { token }.create_response("logged in as admin successfully");
    Ok(response)
}

#[cfg(test)]
mod test {
    use axum_test_helper::TestClient;

    use rstest::*;

    use axum::http::StatusCode;

    use crate::app;

    use super::*;

    #[fixture]
    #[once]
    fn client() -> TestClient {
        let db = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async move { crate::connect_to_database().await })
                .unwrap()
        });

        let app = routes().with_state(app::State::new(db));

        TestClient::new(app)
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
        #[notrace] client: &TestClient,
        #[case] email: String,
        #[case] password: String,
    ) {
        let auth_payload = AuthPayload { email, password };

        let response = client
            .post("/admins/login")
            .json(&auth_payload)
            .send()
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let _ = response.json::<AppResponse<AuthBody>>().await;
    }
}
