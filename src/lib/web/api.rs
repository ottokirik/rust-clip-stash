use std::str::FromStr;

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use rocket::{serde::json::Json, Responder};
use serde::Serialize;

use crate::service::ServiceError;

pub const API_KEY_HEADER: &str = "x-api-key";

#[derive(Responder, Debug, thiserror::Error, Serialize)]
pub enum ApiKeyError {
    #[error("API key not found")]
    #[response(status = 404, content_type = "json")]
    NoyFound(String),

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
