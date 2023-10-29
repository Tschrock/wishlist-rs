-- Add migration script here
CREATE TABLE lists (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL
);
