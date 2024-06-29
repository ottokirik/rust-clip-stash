use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::domain::clip::ClipError;
use crate::domain::time::Time;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExpiresAt(Option<Time>);

impl ExpiresAt {
    pub fn new<T: Into<Option<Time>>>(expires_at: T) -> Self {
        Self(expires_at.into())
    }

    pub fn into_inner(self) -> Option<Time> {
        self.0
    }
}

impl Default for ExpiresAt {
    fn default() -> Self {
        Self::new(None)
    }
}

impl FromStr for ExpiresAt {
    type Err = ClipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Ok(Self(None))
        } else {
            match Time::from_str(s) {
                Ok(time) => Ok(Self::new(time)),
                Err(e) => Err(e.into()),
            }
        }
    }
}
