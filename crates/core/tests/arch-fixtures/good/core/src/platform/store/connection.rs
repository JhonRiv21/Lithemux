use rusqlite::Connection;

pub struct Db {
    pub conn: Connection,
}
