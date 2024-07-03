use rocket::FromForm;
use serde::Serialize;

use crate::domain::clip::field;

#[derive(Debug, Serialize, FromForm)]
pub struct NewClip {
    pub content: field::Content,
    pub title: field::Title,
    pub expires_at: field::ExpiresAt,
    pub password: field::Password,
}

#[derive(Debug, Serialize, FromForm)]
pub struct PasswordProtectedClip {
    pub password: field::Password,
}
