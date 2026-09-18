use std::collections::HashMap;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, Result};

pub const SCHEMA_VERSION: i64 = 3;

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
    updated_at TEXT NOT NULL,
    group_id INTEGER
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
    name TEXT NOT NULL,
    is_workday INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entries_user_date ON entries(user_id, work_date);
CREATE INDEX IF NOT EXISTS idx_entries_updated ON entries(updated_at);
CREATE INDEX IF NOT EXISTS idx_entries_status ON entries(status_id);
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
    migrate(conn)?;
    seed(conn)?;
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
        ("default_status_id", default_status_id.to_string()),
        ("sprint_length_days", "14".to_string()),
        ("reminder_range", "week".to_string()),
        ("language", "zh-TW".to_string()),
        ("theme", "dark".to_string()),
        ("density", "cozy".to_string()),
    ];
    for (key, value) in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO app_settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
    }
    Ok(())
}

fn has_column(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        if row.get::<_, String>(1)? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn table_exists(conn: &Connection, name: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        rusqlite::params![name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn root_of(parent: &mut HashMap<i64, i64>, node: i64) -> i64 {
    let mut current = node;
    loop {
        let next = *parent.entry(current).or_insert(current);
        if next == current {
            break;
        }
        current = next;
    }
    let mut walk = node;
    while walk != current {
        let next = parent[&walk];
        parent.insert(walk, current);
        walk = next;
    }
    current
}

fn convert_links_to_groups(conn: &Connection) -> Result<()> {
    let edges: Vec<(i64, i64)> = {
        let mut stmt = conn.prepare("SELECT entry_id, linked_entry_id FROM entry_links")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<Result<Vec<_>>>()?
    };

    let mut parent: HashMap<i64, i64> = HashMap::new();
    for (left, right) in &edges {
        let a = root_of(&mut parent, *left);
        let b = root_of(&mut parent, *right);
        if a != b {
            parent.insert(a, b);
        }
    }

    let mut groups: HashMap<i64, Vec<i64>> = HashMap::new();
    for node in parent.keys().copied().collect::<Vec<i64>>() {
        let root = root_of(&mut parent, node);
        groups.entry(root).or_default().push(node);
    }

    let mut next_group = 1i64;
    for members in groups.values() {
        if members.len() < 2 {
            continue;
        }
        for member in members {
            conn.execute(
                "UPDATE entries SET group_id = ?2 WHERE id = ?1",
                rusqlite::params![member, next_group],
            )?;
        }
        next_group += 1;
    }
    Ok(())
}

fn migrate(conn: &Connection) -> Result<()> {
    if !has_column(conn, "holidays", "is_workday")? {
        conn.execute(
            "ALTER TABLE holidays ADD COLUMN is_workday INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    if !has_column(conn, "entries", "group_id")? {
        conn.execute("ALTER TABLE entries ADD COLUMN group_id INTEGER", [])?;
    }

    if table_exists(conn, "entry_links")? {
        convert_links_to_groups(conn)?;
        conn.execute("DROP TABLE entry_links", [])?;
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_entries_group ON entries(group_id)",
        [],
    )?;

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
    fn migration_repairs_a_database_whose_version_already_claims_to_be_current() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE holidays (date TEXT PRIMARY KEY, name TEXT NOT NULL);
             CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO app_settings (key, value) VALUES ('schema_version', '2');
             INSERT INTO holidays (date, name) VALUES ('2026-01-01', '開國紀念日');",
        )
        .unwrap();

        prepare(&conn).unwrap();

        assert!(has_column(&conn, "holidays", "is_workday").unwrap());
        let is_workday: i64 = conn
            .query_row(
                "SELECT is_workday FROM holidays WHERE date = '2026-01-01'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(is_workday, 0);
    }

    #[test]
    fn migration_upgrades_a_v2_database_and_converts_links_to_groups() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE entries (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 user_id INTEGER,
                 title TEXT NOT NULL,
                 content TEXT,
                 ticket TEXT,
                 status_id INTEGER NOT NULL,
                 work_date TEXT NOT NULL,
                 hours REAL NOT NULL DEFAULT 0,
                 created_at TEXT NOT NULL,
                 updated_at TEXT NOT NULL
             );
             CREATE TABLE entry_links (
                 entry_id INTEGER NOT NULL,
                 linked_entry_id INTEGER NOT NULL,
                 PRIMARY KEY (entry_id, linked_entry_id)
             );
             CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO app_settings (key, value) VALUES ('schema_version', '2');
             INSERT INTO entries (id, title, status_id, work_date, created_at, updated_at)
                 VALUES (1, '甲', 1, '2026-09-18', 'x', 'x'),
                        (2, '乙', 1, '2026-09-18', 'x', 'x'),
                        (3, '丙', 1, '2026-09-18', 'x', 'x'),
                        (4, '丁', 1, '2026-09-18', 'x', 'x');
             INSERT INTO entry_links (entry_id, linked_entry_id) VALUES (1, 2), (2, 3);",
        )
        .unwrap();

        prepare(&conn).unwrap();

        assert!(!table_exists(&conn, "entry_links").unwrap());

        let group_of = |id: i64| -> Option<i64> {
            conn.query_row(
                "SELECT group_id FROM entries WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .unwrap()
        };

        let first = group_of(1).expect("甲應該有群組");
        assert_eq!(group_of(2), Some(first));
        assert_eq!(group_of(3), Some(first));
        assert_eq!(group_of(4), None);
    }

    #[test]
    fn migration_is_idempotent() {
        let conn = open_in_memory().unwrap();
        prepare(&conn).unwrap();
        prepare(&conn).unwrap();
        assert!(has_column(&conn, "holidays", "is_workday").unwrap());
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
