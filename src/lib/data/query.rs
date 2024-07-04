use sqlx::Row;

use crate::data::{DataError, DatabasePool};
use crate::ShortCode;

use super::model;

type Result<T> = std::result::Result<T, DataError>;

pub async fn increase_hit_count(
    short_code: &ShortCode,
    hits: u32,
    pool: &DatabasePool,
) -> Result<()> {
    let short_code = short_code.as_str();

    Ok(sqlx::query!(
        "UPDATE clips SET hits = hits + ? WHERE short_code = ?",
        hits,
        short_code
    )
    .execute(pool)
    .await
    .map(|_| ())?)
}

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

pub async fn new_clip<M: Into<model::NewClip>>(
    model: M,
    pool: &DatabasePool,
) -> Result<model::Clip> {
    let model = model.into();
    let _ = sqlx::query!(
        r#"INSERT INTO clips (
            clip_id, 
            short_code, 
            content, 
            title, 
            posted_at, 
            expires_at, 
            password, 
            hits) 
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        model.clip_id,
        model.short_code,
        model.content,
        model.title,
        model.posted_at,
        model.expires_at,
        model.password,
        0,
    )
    .execute(pool)
    .await?;

    get_clip(model.short_code, pool).await
}

pub async fn update_clip<M: Into<model::UpdateClip>>(
    model: M,
    pool: &DatabasePool,
) -> Result<model::Clip> {
    let model = model.into();
    let _ = sqlx::query!(
        r#"UPDATE clips SET
            content = ?, 
            title = ?, 
            expires_at = ?, 
            password = ?
           WHERE short_code = ?"#,
        model.content,
        model.title,
        model.expires_at,
        model.password,
        model.short_code,
    )
    .execute(pool)
    .await?;

    get_clip(model.short_code, pool).await
}
