-- Remove 'created_at' and 'updated_at' columns to the 'lists' and 'items' tables
ALTER TABLE lists DROP COLUMN created_at;
ALTER TABLE lists DROP COLUMN updated_at;
ALTER TABLE items DROP COLUMN created_at;
ALTER TABLE items DROP COLUMN updated_at;
