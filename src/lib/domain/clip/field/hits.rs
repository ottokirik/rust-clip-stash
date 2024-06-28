use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Constructor)]
pub struct Hits(u64);

impl Hits {
    pub fn into_inner(self) -> u64 {
        self.0
    }
}
