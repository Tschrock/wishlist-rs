-- SQLite: Add 'created_at' and 'updated_at' columns to the 'lists' and 'items' tables
ALTER TABLE lists ADD COLUMN created_at INTEGER NOT NULL;
ALTER TABLE lists ADD COLUMN updated_at INTEGER NOT NULL;
ALTER TABLE items ADD COLUMN created_at INTEGER NOT NULL;
ALTER TABLE items ADD COLUMN updated_at INTEGER NOT NULL;
