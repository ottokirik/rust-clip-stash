use chrono::{NaiveDateTime, Utc};
use std::convert::TryFrom;

use crate::data::DbId;
use crate::{ClipError, ShortCode, Time};

#[derive(Debug, sqlx::FromRow)]
pub struct Clip {
    pub(in crate::data) clip_id: String,
    pub(in crate::data) short_code: String,
    pub(in crate::data) content: String,
    pub(in crate::data) title: Option<String>,
    pub(in crate::data) posted_at: NaiveDateTime,
    pub(in crate::data) expires_at: Option<NaiveDateTime>,
    pub(in crate::data) password: Option<String>,
    pub(in crate::data) hits: i64,
}

impl TryFrom<Clip> for crate::domain::Clip {
    type Error = ClipError;

    fn try_from(clip: Clip) -> Result<Self, Self::Error> {
        use crate::domain::clip::field;
        use std::str::FromStr;

        Ok(Self {
            clip_id: field::ClipId::new(DbId::from_str(clip.clip_id.as_str())?),
            short_code: field::ShortCode::from(clip.short_code),
            content: field::Content::new(clip.content.as_str())?,
            title: field::Title::new(clip.title),
            posted_at: field::PostedAt::new(Time::form_naive_utc(clip.posted_at)),
            expires_at: field::ExpiresAt::new(clip.expires_at.map(Time::form_naive_utc)),
            password: field::Password::new(clip.password.unwrap_or_default())?,
            hits: field::Hits::new(u64::try_from(clip.hits)?),
        })
    }
}

pub struct GetClip {
    pub(in crate::data) short_code: String,
}

impl From<ShortCode> for GetClip {
    fn from(short_code: ShortCode) -> Self {
        GetClip {
            short_code: short_code.into_inner(),
        }
    }
}

impl From<String> for GetClip {
    fn from(short_code: String) -> Self {
        GetClip { short_code }
    }
}

pub struct NewClip {
    pub(in crate::data) clip_id: String,
    pub(in crate::data) short_code: String,
    pub(in crate::data) content: String,
    pub(in crate::data) title: Option<String>,
    pub(in crate::data) posted_at: i64,
    pub(in crate::data) expires_at: Option<NaiveDateTime>,
    pub(in crate::data) password: Option<String>,
}

pub struct UpdateClip {
    pub(in crate::data) short_code: String,
    pub(in crate::data) content: String,
    pub(in crate::data) title: Option<String>,
    pub(in crate::data) expires_at: Option<i64>,
    pub(in crate::data) password: Option<String>,
}
