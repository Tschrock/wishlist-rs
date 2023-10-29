-- Add migration script here
CREATE TABLE items (
    id BIGSERIAL PRIMARY KEY,
    list_id BIGINT NOT NULL REFERENCES lists(id),
    title TEXT NOT NULL,
    description TEXT,
    price TEXT
);
