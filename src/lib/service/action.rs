use crate::{
    data::{query, DatabasePool, Transaction},
    domain::Clip,
    ShortCode,
};

use super::{ask, ServiceError};

type Result<T> = std::result::Result<T, ServiceError>;
type ResultClip = Result<Clip>;

pub async fn new_clip(req: ask::NewClip, pool: &DatabasePool) -> ResultClip {
    Ok(query::new_clip(req, pool).await?.try_into()?)
}

pub async fn update_clip(req: ask::UpdateClip, pool: &DatabasePool) -> ResultClip {
    Ok(query::update_clip(req, pool).await?.try_into()?)
}

pub async fn get_clip(req: ask::GetClip, pool: &DatabasePool) -> ResultClip {
    let user_password = req.password.clone();
    let clip: Clip = query::get_clip(req, pool).await?.try_into()?;

    if clip.password.has_password() {
        if clip.password == user_password {
            Ok(clip)
        } else {
            Err(ServiceError::PermissionError("invalid password".to_owned()))
        }
    } else {
        Ok(clip)
    }
}

pub async fn increase_hit_count(
    short_code: &ShortCode,
    hits: u32,
    pool: &DatabasePool,
) -> Result<()> {
    Ok(query::increase_hit_count(short_code, hits, pool).await?)
}

pub async fn begin_transaction(pool: &DatabasePool) -> Result<Transaction<'_>> {
    Ok(pool.begin().await?)
}

pub async fn end_transaction(transaction: Transaction<'_>) -> Result<()> {
    Ok(transaction.commit().await?)
}
