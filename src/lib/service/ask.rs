use serde::{Deserialize, Serialize};

use crate::{domain::clip::field, ShortCode};

#[derive(Debug, Deserialize, Serialize)]
pub struct NewClip {
    pub content: field::Content,
    pub title: field::Title,
    pub expires_at: field::ExpiresAt,
    pub password: field::Password,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateClip {
    pub content: field::Content,
    pub title: field::Title,
    pub expires_at: field::ExpiresAt,
    pub password: field::Password,
    pub short_code: field::ShortCode,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetClip {
    pub short_code: ShortCode,
    pub password: field::Password,
}

impl GetClip {
    pub fn from_raw(short_code: &str) -> Self {
        Self {
            short_code: ShortCode::from(short_code),
            password: field::Password::default(),
        }
    }
}

impl From<ShortCode> for GetClip {
    fn from(short_code: ShortCode) -> Self {
        Self {
            short_code,
            password: field::Password::default(),
        }
    }
}

impl From<&str> for GetClip {
    fn from(raw: &str) -> Self {
        Self::from_raw(raw)
    }
}
