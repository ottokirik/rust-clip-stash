use derive_more::Constructor;
use serde::{Deserialize, Serialize};

use crate::domain::time::Time;

#[derive(Clone, Debug, Deserialize, Serialize, Constructor)]
pub struct PostedAt(Time);

impl PostedAt {
    pub fn into_inner(self) -> Time {
        self.0
    }
}
