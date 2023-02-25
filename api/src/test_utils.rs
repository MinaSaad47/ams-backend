#![cfg(test)]

use axum::{async_trait, Router};
use hyper::{header, Body, Method, Request, StatusCode};
use mockall::mock;
use serde::{de::DeserializeOwned, Serialize};
use tower::{Service, ServiceExt};
use uuid::Uuid;

use common::prelude::*;
use logic::prelude::*;

#[derive(Debug)]
pub struct TestingApp {
    router: Router,
    base_url: String,
}

impl TestingApp {
    pub fn new(router: Router, base_url: &str) -> Self {
        Self {
            router,
            base_url: base_url.to_owned(),
        }
    }

    pub async fn request<B: DeserializeOwned, U: Serialize>(
        &mut self,
        url: &str,
        method: Method,
        body: Option<U>,
    ) -> TestingResponse<B> {
        let body = {
            if let Some(body) = body {
                serde_json::to_vec(&body).unwrap().into()
            } else {
                Body::empty()
            }
        };
        let req = Request::builder()
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .uri(format!("{}{}", self.base_url, url))
            .body(body)
            .unwrap();

        let res = self.router.ready().await.unwrap().call(req).await.unwrap();
        let status = res.status();
        let body = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: Option<Wrapper<B>> = body.try_into().ok();
        TestingResponse {
            status,
            body: body.map(|b| b.0),
        }
    }
}

pub struct TestingResponse<B> {
    pub status: StatusCode,
    pub body: Option<B>,
}

mock! {
    pub AdminRepoStruct {}
    #[async_trait]
    impl AdminsRepoTrait for AdminRepoStruct {
        async fn create(&self, admin: CreateAdmin) -> Result<Admin, RepoError>;
        async fn get_by_id(&self, id: Uuid) -> Result<Admin, RepoError>;
        async fn get_by_email(&self, email: String) -> Result<Admin, RepoError>;
        async fn get_all(&self) -> Result<Vec<Admin>, RepoError>;
        async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
    }
}

mock! {
    pub AttendeeRepoStruct{}
    #[async_trait]
    impl AttendeesRepoTrait for AttendeeRepoStruct {
        async fn create(&self, attendee: CreateAttendee) -> Result<Attendee, RepoError>;
        async fn update(
            &self,
            id: Uuid,
            update_attendee: UpdateAttendee,
        ) -> Result<Attendee, RepoError>;
        async fn get_by_id(&self, id: Uuid) -> Result<Attendee, RepoError>;
        async fn get_by_email(&self, email: String) -> Result<Attendee, RepoError>;
        async fn get_all(&self) -> Result<Vec<Attendee>, RepoError>;
        async fn delete_by_id(&self, id: Uuid) -> Result<(), RepoError>;
    }
}
