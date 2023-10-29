-- Add migration script here
CREATE TABLE lists (
    id BIGSERIAL PRIMARY KEY,
    key TEXT NOT NULL,
    is_private BOOLEAN NOT NULL DEFAULT TRUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL
);
