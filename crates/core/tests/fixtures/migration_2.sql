-- What migration 2 ran (block 3 commit), on top of schema_v1.sql.
ALTER TABLE sessions ADD COLUMN title TEXT;
ALTER TABLE sessions ADD COLUMN pin_sort INTEGER;
ALTER TABLE sessions ADD COLUMN preview TEXT;
ALTER TABLE sessions ADD COLUMN unread INTEGER NOT NULL DEFAULT 0;
ALTER TABLE categories ADD COLUMN collapsed INTEGER NOT NULL DEFAULT 0;

CREATE TABLE workspace_layout(
  id INTEGER PRIMARY KEY CHECK(id = 1),
  layout TEXT NOT NULL);
