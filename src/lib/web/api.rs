use std::str::FromStr;

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    serde::json::Json,
    Responder, State,
};
use serde::Serialize;

use crate::{
    data::AppDatabase,
    service::{action, ServiceError},
};

pub const API_KEY_HEADER: &str = "x-api-key";

#[derive(Responder, Debug, thiserror::Error, Serialize)]
pub enum ApiKeyError {
    #[error("API key not found")]
    #[response(status = 404, content_type = "json")]
    NotFound(String),

    #[error("invalid API key format")]
    #[response(status = 400, content_type = "json")]
    DecodeError(String),
}

#[derive(Debug, Clone)]
pub struct ApiKey(Vec<u8>);

impl ApiKey {
    pub fn to_base64(&self) -> String {
        URL_SAFE.encode(self.0.as_slice())
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl Default for ApiKey {
    fn default() -> Self {
        let key = (0..16).map(|_| rand::random::<u8>()).collect();
        Self(key)
    }
}

impl FromStr for ApiKey {
    type Err = ApiKeyError;

    fn from_str(key: &str) -> Result<Self, Self::Err> {
        URL_SAFE
            .decode(key)
            .map(ApiKey)
            .map_err(|err| Self::Err::DecodeError(err.to_string()))
    }
}

#[derive(Responder, Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found")]
    #[response(status = 404, content_type = "json")]
    NotFound(Json<String>),

    #[error("server error")]
    #[response(status = 500, content_type = "json")]
    Server(Json<String>),

    #[error("client error")]
    #[response(status = 400, content_type = "json")]
    User(Json<String>),

    #[error("key error")]
    #[response(status = 400, content_type = "json")]
    KeyError(Json<ApiKeyError>),
}

impl From<ServiceError> for ApiError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::Clip(err) => Self::User(Json(format!("clip parsing error: {}", err))),
            ServiceError::NotFound => Self::NotFound(Json("not found".to_owned())),
            ServiceError::Data(_) => Self::Server(Json("a server error occurred".to_owned())),
            ServiceError::PermissionError(err) => Self::User(Json(err)),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ApiError;

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        fn server_error() -> Outcome<ApiKey, ApiError> {
            Outcome::Error((
                Status::InternalServerError,
                ApiError::Server(Json("server error".to_owned())),
            ))
        }

        fn key_error(err: ApiKeyError) -> Outcome<ApiKey, ApiError> {
            Outcome::Error((Status::BadRequest, ApiError::KeyError(Json(err))))
        }

        match req.headers().get_one(API_KEY_HEADER) {
            None => key_error(ApiKeyError::NotFound("API key not found".to_owned())),
            Some(key) => {
                let db = match req.guard::<&State<AppDatabase>>().await {
                    Outcome::Success(db) => db,
                    _ => return server_error(),
                };

                let api_key = match ApiKey::from_str(key) {
                    Ok(key) => key,
                    Err(err) => return key_error(err),
                };

                match action::is_valid_api_key(api_key.clone(), db.get_pool()).await {
                    Ok(valid) if valid => Outcome::Success(api_key),
                    Ok(valid) if !valid => {
                        key_error(ApiKeyError::NotFound("API key not found".to_owned()))
                    }
                    _ => server_error(),
                }
            }
        }
    }
}
