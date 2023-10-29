-- Add migration script here
CREATE TABLE lists (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL
);
