-- Create lists table
CREATE TABLE lists (
    id INTEGER PRIMARY KEY,
    key TEXT NOT NULL,
    is_private BOOLEAN NOT NULL DEFAULT TRUE,
    title TEXT NOT NULL,
    description TEXT NOT NULL
);
