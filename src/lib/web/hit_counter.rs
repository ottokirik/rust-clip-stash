use crossbeam_channel::Sender;
use tokio::runtime::Handle;

use crate::{data::DatabasePool, ShortCode};

enum HitCountMsg {
    Commit,
    Hit(ShortCode, u32),
}

pub struct HitCounter {
    tx: Sender<HitCountMsg>,
}

impl HitCounter {
    pub fn new(pool: DatabasePool, handle: Handle) -> Self {
        todo!()
    }
}
