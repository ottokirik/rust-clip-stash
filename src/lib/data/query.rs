use sqlx::Row;

use crate::data::{DataError, DatabasePool};
use crate::ShortCode;

use super::model;

type Result<T> = std::result::Result<T, DataError>;

pub async fn get_clip<M: Into<model::GetClip>>(
    model: M,
    pool: &DatabasePool,
) -> Result<model::Clip> {
    let model = model.into();
    let short_code = model.short_code.as_str();

    Ok(sqlx::query_as!(
        model::Clip,
        "SELECT * FROM clips WHERE short_code = ?",
        short_code
    )
    .fetch_one(pool)
    .await?)
}
