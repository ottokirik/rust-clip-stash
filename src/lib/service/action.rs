use crate::{
    data::{query, DatabasePool},
    domain::Clip,
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
