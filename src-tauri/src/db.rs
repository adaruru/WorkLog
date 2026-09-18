use std::path::{Path, PathBuf};

use rusqlite::{Connection, Result};

pub const SCHEMA_VERSION: i64 = 1;

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    daily_required_hours REAL NOT NULL DEFAULT 8,
    is_active INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS statuses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    content TEXT,
    ticket TEXT,
    status_id INTEGER NOT NULL REFERENCES statuses(id),
    work_date TEXT NOT NULL,
    hours REAL NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS entry_links (
    entry_id INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    linked_entry_id INTEGER NOT NULL REFERENCES entries(id) ON DELETE CASCADE,
    PRIMARY KEY (entry_id, linked_entry_id)
);

CREATE TABLE IF NOT EXISTS user_leaves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    leave_date TEXT NOT NULL,
    hours REAL NOT NULL,
    note TEXT
);

CREATE TABLE IF NOT EXISTS holidays (
    date TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entries_user_date ON entries(user_id, work_date);
CREATE INDEX IF NOT EXISTS idx_entries_updated ON entries(updated_at);
CREATE INDEX IF NOT EXISTS idx_entries_status ON entries(status_id);
CREATE INDEX IF NOT EXISTS idx_entry_links_linked ON entry_links(linked_entry_id);
CREATE INDEX IF NOT EXISTS idx_user_leaves_user_date ON user_leaves(user_id, leave_date);
"#;

const BUILTIN_STATUSES: [(&str, &str, i64); 4] = [
    ("new", "#64748b", 0),
    ("active", "#2563eb", 1),
    ("pending", "#d97706", 2),
    ("closed", "#16a34a", 3),
];

pub fn default_db_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("worklog.db")))
        .unwrap_or_else(|| PathBuf::from("worklog.db"))
}

pub fn open(path: &Path) -> Result<Connection> {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let conn = Connection::open(path)?;
    prepare(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    prepare(&conn)?;
    Ok(conn)
}

fn prepare(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.execute_batch(SCHEMA_SQL)?;
    seed(conn)?;
    migrate(conn)?;
    Ok(())
}

fn seed(conn: &Connection) -> Result<()> {
    for (name, color, sort_order) in BUILTIN_STATUSES {
        conn.execute(
            "INSERT OR IGNORE INTO statuses (name, color, is_builtin, sort_order) VALUES (?1, ?2, 1, ?3)",
            rusqlite::params![name, color, sort_order],
        )?;
    }

    let default_status_id: i64 =
        conn.query_row("SELECT id FROM statuses WHERE name = 'new'", [], |row| row.get(0))?;

    let defaults: [(&str, String); 6] = [
        ("schema_version", SCHEMA_VERSION.to_string()),
        ("default_status_id", default_status_id.to_string()),
        ("sprint_length_days", "14".to_string()),
        ("reminder_range", "week".to_string()),
        ("language", "zh-TW".to_string()),
        ("theme", "dark".to_string()),
    ];
    for (key, value) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO app_settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
    }
    Ok(())
}

fn migrate(conn: &Connection) -> Result<()> {
    let current: i64 = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = 'schema_version'",
            [],
            |row| row.get::<_, String>(0),
        )
        .map(|value| value.parse().unwrap_or(SCHEMA_VERSION))
        .unwrap_or(SCHEMA_VERSION);

    if current >= SCHEMA_VERSION {
        return Ok(());
    }

    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('schema_version', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![SCHEMA_VERSION.to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_four_builtin_statuses() {
        let conn = open_in_memory().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM statuses WHERE is_builtin = 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn default_status_points_to_new() {
        let conn = open_in_memory().unwrap();
        let default_id: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'default_status_id'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let new_id: i64 = conn
            .query_row("SELECT id FROM statuses WHERE name = 'new'", [], |row| row.get(0))
            .unwrap();
        assert_eq!(default_id, new_id.to_string());
    }

    #[test]
    fn re_running_prepare_does_not_duplicate_statuses() {
        let conn = open_in_memory().unwrap();
        prepare(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM statuses", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 4);
    }
}
