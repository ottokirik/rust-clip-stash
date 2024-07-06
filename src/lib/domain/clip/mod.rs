pub mod field;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClipError {
    #[error("invalid password: {0}")]
    InvalidPassword(String),
    #[error("invalid title: {0}")]
    InvalidTitle(String),
    #[error("empty content")]
    EmptyContent,
    #[error("invalid date: {0}")]
    InvalidDate(String),
    #[error("date parse error: {0}")]
    DateParse(#[from] chrono::ParseError),
    #[error("id parse error: {0}")]
    Id(#[from] uuid::Error),
    #[error("hits parse error: {0}")]
    Hits(#[from] std::num::TryFromIntError),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Clip {
    #[serde(skip)]
    pub clip_id: field::ClipId,
    pub short_code: field::ShortCode,
    pub content: field::Content,
    pub title: field::Title,
    pub posted_at: field::PostedAt,
    pub expires_at: field::ExpiresAt,
    pub password: field::Password,
    pub hits: field::Hits,
}
