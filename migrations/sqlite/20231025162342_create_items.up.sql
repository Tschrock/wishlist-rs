-- Create items table
CREATE TABLE items (
    id INTEGER PRIMARY KEY,
    list_id INTEGER NOT NULL REFERENCES lists(id),
    title TEXT NOT NULL,
    description TEXT,
    price TEXT
);
