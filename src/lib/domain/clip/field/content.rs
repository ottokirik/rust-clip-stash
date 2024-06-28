use serde::{Deserialize, Serialize};

use crate::domain::clip::ClipError;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Content(String);

impl Content {
    pub fn new(content: &str) -> Result<Self, ClipError> {
        match content.trim().is_empty() {
            true => Err(ClipError::EmptyContent),
            false => Ok(Content(content.to_owned())),
        }
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
