-- The schema of a version 3 database exactly as SQLite keeps it after the
-- app's own migrations 1 to 3 (dumped from a database made by the app).
CREATE TABLE categories(
  id TEXT PRIMARY KEY,
  slug TEXT UNIQUE NOT NULL,
  name TEXT NOT NULL,
  root_dir TEXT NOT NULL,
  color TEXT,
  sort INTEGER,
  created_at INTEGER, collapsed INTEGER NOT NULL DEFAULT 0);
CREATE TABLE sessions(
  id TEXT PRIMARY KEY,
  category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
  slug TEXT NOT NULL,
  role TEXT NOT NULL,
  kind TEXT NOT NULL CHECK(kind IN ('mother','child')),
  profile TEXT NOT NULL CHECK(profile IN ('claude','shell')),
  cwd TEXT,
  claude_session_id TEXT,
  model TEXT,
  keep_alive INTEGER NOT NULL DEFAULT 0,
  state TEXT NOT NULL DEFAULT 'new',
  claude_version TEXT,
  last_activity_at INTEGER,
  created_at INTEGER,
  sort INTEGER, title TEXT, pin_sort INTEGER, preview TEXT, unread INTEGER NOT NULL DEFAULT 0, notice TEXT,
  UNIQUE(category_id, slug));
CREATE UNIQUE INDEX one_mother_per_category ON sessions(category_id) WHERE kind='mother';
CREATE TABLE settings(key TEXT PRIMARY KEY, value TEXT);
CREATE TABLE workspace_layout(
  id INTEGER PRIMARY KEY CHECK(id = 1),
  layout TEXT NOT NULL);
