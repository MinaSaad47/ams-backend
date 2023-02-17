use axum::{http::StatusCode, response::IntoResponse, Json};
use sea_orm::sea_query::tests_cfg::json;
use serde::Serialize;

#[derive(Debug)]
pub struct AppResponse<Data>
where
    Data: Serialize,
{
    pub code: StatusCode,
    pub message: String,
    pub data: Option<Data>,
}

impl<Data> AppResponse<Data>
where
    Data: Serialize,
{
    pub fn created(data: Data, message: &str) -> Self {
        Self {
            code: StatusCode::CREATED,
            message: message.to_owned(),
            data: Some(data),
        }
    }
    pub fn no_content(message: &str) -> Self {
        Self {
            code: StatusCode::OK,
            message: message.to_owned(),
            data: None,
        }
    }
    pub fn with_content(data: Data, message: &str) -> Self {
        Self {
            code: StatusCode::OK,
            message: message.to_owned(),
            data: Some(data),
        }
    }
}

impl<Data> IntoResponse for AppResponse<Data>
where
    Data: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        match self.data {
            Some(data) => {
                Json(json!({"code": self.code.as_u16(), "status": true, "message": self.message, "data": data}))
                    .into_response()
            }
            None =>  {

                Json(json!({"code": self.code.as_u16(), "status": true, "message": self.message}))
                    .into_response()
            }
        }
    }
}
