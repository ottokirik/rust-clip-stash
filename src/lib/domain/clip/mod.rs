pub mod field;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Clip {
    pub clip_id: field::ClipId,
    pub short_code: field::ShortCode,
    pub content: field::Content,
    pub title: field::Title,
    pub posted_at: field::PostedAt,
    pub expires_at: field::ExpiresAt,
    pub password: field::Password,
    pub hits: field::Hits,
}
