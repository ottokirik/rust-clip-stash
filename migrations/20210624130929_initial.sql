-- Add migration script here
CREATE TABLE IF NOT EXISTS clips (
    clip_id TEXT PRIMARY KEY NOT NULL,
    short_code TEXT UNIQUE NOT NULL,
    content TEXT NOT NULL,
    title TEXT,
    posted_at DATETIME NOT NULL,
    expires_at DATETIME,
    password TEXT,
    hits BIGINT NOT NULL
);