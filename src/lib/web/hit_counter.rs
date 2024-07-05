use std::{collections::HashMap, sync::Arc};

use crossbeam_channel::{unbounded, Sender};
use parking_lot::Mutex;
use tokio::runtime::Handle;

use crate::{
    data::DatabasePool,
    service::{self, ServiceError},
    ShortCode,
};

type HitStore = Arc<Mutex<HashMap<ShortCode, u32>>>;
type Result<T> = std::result::Result<T, HitCountError>;

#[derive(Debug, thiserror::Error)]
enum HitCountError {
    #[error("service error: {0}")]
    Service(#[from] ServiceError),
    #[error("communication error: {0}")]
    Channel(#[from] crossbeam_channel::SendError<HitCountMsg>),
}

enum HitCountMsg {
    Commit,
    Hit(ShortCode, u32),
}

pub struct HitCounter {
    tx: Sender<HitCountMsg>,
}

impl HitCounter {
    pub fn new(pool: DatabasePool, handle: Handle) -> Self {
        let (tx, rx) = unbounded();
        let tx_clone = tx.clone();
        let rx_clone = rx.clone();

        let _ = std::thread::spawn(move || {
            println!("HitCounter thread spawned");

            let store: HitStore = Arc::new(Mutex::new(HashMap::new()));

            loop {
                match rx_clone.try_recv() {
                    Ok(msg) => {
                        if let Err(err) =
                            Self::process_msg(msg, store.clone(), handle.clone(), pool.clone())
                        {
                            eprintln!("failed to process message: {}", err);
                        }
                    }
                    Err(err) => match err {
                        crossbeam_channel::TryRecvError::Empty => {
                            std::thread::sleep(std::time::Duration::from_secs(5));

                            if let Err(err) = tx_clone.send(HitCountMsg::Commit) {
                                eprintln!("failed to send commit message: {}", err);
                            }
                        }
                        _ => break,
                    },
                }
            }
        });

        Self { tx }
    }

    fn process_msg(
        msg: HitCountMsg,
        hits: HitStore,
        handle: Handle,
        pool: DatabasePool,
    ) -> Result<()> {
        match msg {
            HitCountMsg::Commit => Self::commit_hits(hits.clone(), handle.clone(), pool.clone())?,
            HitCountMsg::Hit(short_code, count) => {
                let mut hit_count = hits.lock();
                let hit_count = hit_count.entry(short_code).or_insert(0);
                *hit_count += count;
            }
        }

        Ok(())
    }

    fn commit_hits(hits: HitStore, handle: Handle, pool: DatabasePool) -> Result<()> {
        let hits = Arc::clone(&hits);

        let hits: Vec<(ShortCode, u32)> = {
            let mut hits = hits.lock();
            let hits_vec = hits.iter().map(|(k, v)| (k.clone(), *v)).collect();
            hits.clear();
            hits_vec
        };

        handle.block_on(async move {
            let transaction = service::action::begin_transaction(&pool).await?;

            for (short_code, count) in hits {
                if let Err(err) =
                    service::action::increase_hit_count(&short_code, count, &pool).await
                {
                    eprintln!("failed to increase hit count: {}", err);
                }
            }

            Ok(service::action::end_transaction(transaction).await?)
        })
    }

    pub fn hit(&self, short_code: ShortCode, count: u32) {
        if let Err(err) = self.tx.send(HitCountMsg::Hit(short_code, count)) {
            eprintln!("failed to send hit count: {}", err);
        }
    }
}
