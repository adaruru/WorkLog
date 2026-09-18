use std::collections::HashMap;

use chrono::{Duration, Local, NaiveDate};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Result};

use crate::calendar::{self, SprintConfig};
use crate::models::*;
use crate::query;

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn all_settings(conn: &Connection) -> Result<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM app_settings")?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
    let mut map = HashMap::new();
    for row in rows {
        let (key, value) = row?;
        map.insert(key, value);
    }
    Ok(map)
}

fn default_status_id(conn: &Connection) -> Result<i64> {
    if let Some(value) = get_setting(conn, "default_status_id")? {
        if let Ok(id) = value.parse::<i64>() {
            let exists: Option<i64> = conn
                .query_row("SELECT id FROM statuses WHERE id = ?1", params![id], |row| {
                    row.get(0)
                })
                .optional()?;
            if let Some(id) = exists {
                return Ok(id);
            }
        }
    }
    conn.query_row("SELECT id FROM statuses ORDER BY sort_order LIMIT 1", [], |row| {
        row.get(0)
    })
}

pub fn list_users(conn: &Connection) -> Result<Vec<User>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, daily_required_hours, is_active, sort_order
         FROM users ORDER BY created_at, id",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(User {
            id: row.get(0)?,
            name: row.get(1)?,
            daily_required_hours: row.get(2)?,
            is_active: row.get::<_, i64>(3)? != 0,
            sort_order: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn create_user(conn: &Connection, name: &str, daily_required_hours: f64) -> Result<i64> {
    let next_order: i64 = conn.query_row(
        "SELECT IFNULL(MAX(sort_order), -1) + 1 FROM users",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO users (name, daily_required_hours, is_active, sort_order, created_at)
         VALUES (?1, ?2, 1, ?3, ?4)",
        params![name, daily_required_hours, next_order, now()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_user(
    conn: &Connection,
    id: i64,
    name: &str,
    daily_required_hours: f64,
    is_active: bool,
) -> Result<()> {
    conn.execute(
        "UPDATE users SET name = ?2, daily_required_hours = ?3, is_active = ?4 WHERE id = ?1",
        params![id, name, daily_required_hours, is_active as i64],
    )?;
    Ok(())
}

pub fn delete_user(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM users WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn list_statuses(conn: &Connection) -> Result<Vec<Status>> {
    let mut stmt = conn
        .prepare("SELECT id, name, color, is_builtin, sort_order FROM statuses ORDER BY sort_order, id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Status {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            is_builtin: row.get::<_, i64>(3)? != 0,
            sort_order: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn create_status(conn: &Connection, name: &str, color: &str) -> Result<i64> {
    let next_order: i64 = conn.query_row(
        "SELECT IFNULL(MAX(sort_order), -1) + 1 FROM statuses",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO statuses (name, color, is_builtin, sort_order) VALUES (?1, ?2, 0, ?3)",
        params![name, color, next_order],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_status(conn: &Connection, id: i64, name: &str, color: &str) -> Result<()> {
    conn.execute(
        "UPDATE statuses SET name = ?2, color = ?3 WHERE id = ?1",
        params![id, name, color],
    )?;
    Ok(())
}

pub fn delete_status(conn: &Connection, id: i64, replacement_id: i64) -> Result<()> {
    if id == replacement_id {
        return Err(rusqlite::Error::InvalidParameterName(
            "replacement must differ from the status being deleted".into(),
        ));
    }
    if default_status_id(conn)? == id {
        return Err(rusqlite::Error::InvalidParameterName(
            "the default status cannot be deleted".into(),
        ));
    }
    conn.execute(
        "UPDATE entries SET status_id = ?2 WHERE status_id = ?1",
        params![id, replacement_id],
    )?;
    conn.execute("DELETE FROM statuses WHERE id = ?1", params![id])?;
    Ok(())
}

const ENTRY_COLUMNS: &str = "SELECT e.id, e.user_id, u.name, e.title, IFNULL(e.content, ''), \
     IFNULL(e.ticket, ''), e.status_id, s.name, s.color, e.work_date, e.hours, \
     e.created_at, e.updated_at \
     FROM entries e \
     JOIN statuses s ON s.id = e.status_id \
     LEFT JOIN users u ON u.id = e.user_id";

fn map_entry(row: &rusqlite::Row) -> Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        user_id: row.get(1)?,
        user_name: row.get(2)?,
        title: row.get(3)?,
        content: row.get(4)?,
        ticket: row.get(5)?,
        status_id: row.get(6)?,
        status_name: row.get(7)?,
        status_color: row.get(8)?,
        work_date: row.get(9)?,
        hours: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

pub fn search_entries(conn: &Connection, filter: &EntryQuery) -> Result<Vec<Entry>> {
    let (where_sql, params) = query::build_filter(filter);
    let sql = format!("{ENTRY_COLUMNS}{where_sql}{}", query::ORDER_BY);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(params.iter()), map_entry)?;
    rows.collect()
}

pub fn get_entry(conn: &Connection, id: i64) -> Result<Option<Entry>> {
    let sql = format!("{ENTRY_COLUMNS} WHERE e.id = ?1");
    conn.query_row(&sql, params![id], map_entry).optional()
}

pub fn save_entry(conn: &Connection, input: &EntryInput) -> Result<i64> {
    let status_id = match input.status_id {
        Some(id) => id,
        None => default_status_id(conn)?,
    };
    match input.id {
        Some(id) => {
            conn.execute(
                "UPDATE entries SET user_id = ?2, title = ?3, content = ?4, ticket = ?5,
                 status_id = ?6, work_date = ?7, hours = ?8, updated_at = ?9 WHERE id = ?1",
                params![
                    id,
                    input.user_id,
                    input.title,
                    input.content,
                    input.ticket,
                    status_id,
                    input.work_date,
                    input.hours,
                    now()
                ],
            )?;
            Ok(id)
        }
        None => {
            let stamp = now();
            conn.execute(
                "INSERT INTO entries (user_id, title, content, ticket, status_id, work_date, hours, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8)",
                params![
                    input.user_id,
                    input.title,
                    input.content,
                    input.ticket,
                    status_id,
                    input.work_date,
                    input.hours,
                    stamp
                ],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }
}

pub fn set_entry_status(conn: &Connection, id: i64, status_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE entries SET status_id = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, status_id, now()],
    )?;
    Ok(())
}

pub fn delete_entry(conn: &Connection, id: i64) -> Result<()> {
    let group = group_of(conn, id)?;
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    if let Some(group) = group {
        dissolve_if_alone(conn, group)?;
    }
    Ok(())
}

const REF_COLUMNS: &str = "SELECT e.id, e.title, IFNULL(e.ticket, ''), e.work_date, s.name, s.color, e.updated_at \
     FROM entries e JOIN statuses s ON s.id = e.status_id";

fn map_ref(row: &rusqlite::Row) -> Result<EntryRef> {
    Ok(EntryRef {
        id: row.get(0)?,
        title: row.get(1)?,
        ticket: row.get(2)?,
        work_date: row.get(3)?,
        status_name: row.get(4)?,
        status_color: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

pub fn entry_detail(conn: &Connection, id: i64) -> Result<Option<EntryDetail>> {
    let Some(entry) = get_entry(conn, id)? else {
        return Ok(None);
    };

    let related = match group_of(conn, id)? {
        Some(group) => {
            let sql = format!(
                "{REF_COLUMNS} WHERE e.group_id = ?1 AND e.id <> ?2 ORDER BY e.updated_at DESC"
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt
                .query_map(params![group, id], map_ref)?
                .collect::<Result<Vec<_>>>()?;
            rows
        }
        None => Vec::new(),
    };

    Ok(Some(EntryDetail { entry, related }))
}

fn group_of(conn: &Connection, id: i64) -> Result<Option<i64>> {
    conn.query_row(
        "SELECT group_id FROM entries WHERE id = ?1",
        params![id],
        |row| row.get::<_, Option<i64>>(0),
    )
    .optional()
    .map(|value| value.flatten())
}

fn set_group(conn: &Connection, id: i64, group: Option<i64>) -> Result<()> {
    conn.execute(
        "UPDATE entries SET group_id = ?2 WHERE id = ?1",
        params![id, group],
    )?;
    Ok(())
}

fn dissolve_if_alone(conn: &Connection, group: i64) -> Result<()> {
    let remaining: i64 = conn.query_row(
        "SELECT COUNT(*) FROM entries WHERE group_id = ?1",
        params![group],
        |row| row.get(0),
    )?;
    if remaining <= 1 {
        conn.execute(
            "UPDATE entries SET group_id = NULL WHERE group_id = ?1",
            params![group],
        )?;
    }
    Ok(())
}

pub fn link_candidates(conn: &Connection, entry_id: i64, keyword: &str) -> Result<Vec<EntryRef>> {
    let keyword = keyword.trim();
    let like = format!("%{}%", query::escape_like(keyword));
    let group = group_of(conn, entry_id)?;
    let sql = format!(
        "{REF_COLUMNS} WHERE e.id <> ?1
         AND (?2 IS NULL OR e.group_id IS NULL OR e.group_id <> ?2)
         AND (?3 = '' OR e.title LIKE ?4 ESCAPE '\\' OR IFNULL(e.ticket, '') LIKE ?4 ESCAPE '\\')
         ORDER BY e.updated_at DESC LIMIT 50"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![entry_id, group, keyword, like], map_ref)?;
    rows.collect()
}

pub fn link_entries(conn: &Connection, entry_id: i64, linked_entry_id: i64) -> Result<()> {
    if entry_id == linked_entry_id {
        return Ok(());
    }

    match (group_of(conn, entry_id)?, group_of(conn, linked_entry_id)?) {
        (None, None) => {
            let next: i64 = conn.query_row(
                "SELECT IFNULL(MAX(group_id), 0) + 1 FROM entries",
                [],
                |row| row.get(0),
            )?;
            set_group(conn, entry_id, Some(next))?;
            set_group(conn, linked_entry_id, Some(next))?;
        }
        (Some(group), None) => set_group(conn, linked_entry_id, Some(group))?,
        (None, Some(group)) => set_group(conn, entry_id, Some(group))?,
        (Some(keep), Some(merge)) if keep != merge => {
            conn.execute(
                "UPDATE entries SET group_id = ?1 WHERE group_id = ?2",
                params![keep, merge],
            )?;
        }
        _ => {}
    }
    Ok(())
}

pub fn unlink_entry(conn: &Connection, id: i64) -> Result<()> {
    let Some(group) = group_of(conn, id)? else {
        return Ok(());
    };
    set_group(conn, id, None)?;
    dissolve_if_alone(conn, group)
}

pub fn list_holidays(conn: &Connection) -> Result<Vec<Holiday>> {
    let mut stmt =
        conn.prepare("SELECT date, name, is_workday FROM holidays ORDER BY date DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(Holiday {
            date: row.get(0)?,
            name: row.get(1)?,
            is_workday: row.get::<_, i64>(2)? != 0,
        })
    })?;
    rows.collect()
}

pub fn save_holiday(conn: &Connection, date: &str, name: &str, is_workday: bool) -> Result<()> {
    conn.execute(
        "INSERT INTO holidays (date, name, is_workday) VALUES (?1, ?2, ?3)
         ON CONFLICT(date) DO UPDATE SET name = excluded.name, is_workday = excluded.is_workday",
        params![date, name, is_workday as i64],
    )?;
    Ok(())
}

pub fn import_holidays(conn: &Connection, items: &[Holiday]) -> Result<usize> {
    let tx = conn.unchecked_transaction()?;
    for item in items {
        if calendar::parse_date(item.date.trim()).is_none() {
            continue;
        }
        tx.execute(
            "INSERT INTO holidays (date, name, is_workday) VALUES (?1, ?2, ?3)
             ON CONFLICT(date) DO UPDATE SET name = excluded.name, is_workday = excluded.is_workday",
            params![item.date.trim(), item.name.trim(), item.is_workday as i64],
        )?;
    }
    tx.commit()?;
    Ok(items.len())
}

pub fn clear_holidays_in_year(conn: &Connection, year: i32) -> Result<()> {
    conn.execute(
        "DELETE FROM holidays WHERE date LIKE ?1",
        params![format!("{year}-%")],
    )?;
    Ok(())
}

pub fn delete_holiday(conn: &Connection, date: &str) -> Result<()> {
    conn.execute("DELETE FROM holidays WHERE date = ?1", params![date])?;
    Ok(())
}

pub fn list_leaves(conn: &Connection, user_id: Option<i64>) -> Result<Vec<Leave>> {
    let mut sql = "SELECT l.id, l.user_id, u.name, l.leave_date, l.hours, IFNULL(l.note, '')
         FROM user_leaves l JOIN users u ON u.id = l.user_id"
        .to_string();
    if user_id.is_some() {
        sql.push_str(" WHERE l.user_id = ?1");
    }
    sql.push_str(" ORDER BY l.leave_date DESC");

    let mut stmt = conn.prepare(&sql)?;
    let map = |row: &rusqlite::Row| {
        Ok(Leave {
            id: row.get(0)?,
            user_id: row.get(1)?,
            user_name: row.get(2)?,
            leave_date: row.get(3)?,
            hours: row.get(4)?,
            note: row.get(5)?,
        })
    };
    match user_id {
        Some(id) => stmt.query_map(params![id], map)?.collect(),
        None => stmt.query_map([], map)?.collect(),
    }
}

pub fn save_leave(
    conn: &Connection,
    id: Option<i64>,
    user_id: i64,
    leave_date: &str,
    hours: f64,
    note: &str,
) -> Result<()> {
    match id {
        Some(id) => conn.execute(
            "UPDATE user_leaves SET user_id = ?2, leave_date = ?3, hours = ?4, note = ?5 WHERE id = ?1",
            params![id, user_id, leave_date, hours, note],
        )?,
        None => conn.execute(
            "INSERT INTO user_leaves (user_id, leave_date, hours, note) VALUES (?1, ?2, ?3, ?4)",
            params![user_id, leave_date, hours, note],
        )?,
    };
    Ok(())
}

pub fn delete_leave(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM user_leaves WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn sprint_settings(conn: &Connection) -> Result<SprintSettings> {
    Ok(SprintSettings {
        anchor_number: get_setting(conn, "sprint_anchor_number")?
            .and_then(|value| value.parse().ok()),
        anchor_start_date: get_setting(conn, "sprint_anchor_start_date")?
            .filter(|value| !value.is_empty()),
        length_days: get_setting(conn, "sprint_length_days")?
            .and_then(|value| value.parse().ok())
            .filter(|days| *days > 0)
            .unwrap_or(14),
    })
}

pub fn sprint_config(conn: &Connection) -> Result<Option<SprintConfig>> {
    let settings = sprint_settings(conn)?;
    let (Some(anchor_number), Some(start)) = (
        settings.anchor_number,
        settings
            .anchor_start_date
            .as_deref()
            .and_then(calendar::parse_date),
    ) else {
        return Ok(None);
    };
    Ok(Some(SprintConfig {
        anchor_number,
        anchor_start: start,
        length_days: settings.length_days,
    }))
}

pub struct ResolvedRange {
    pub scope: String,
    pub sprint_number: Option<i64>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

pub fn resolve_range(conn: &Connection, scope: &str, today: NaiveDate) -> Result<ResolvedRange> {
    let mut sprint_number = None;
    let (from, to) = match scope {
        "today" => (Some(today), Some(today)),
        "week" => {
            let (start, end) = calendar::week_range(today);
            (Some(start), Some(end))
        }
        "month" => {
            let (start, end) = calendar::month_range(today);
            (Some(start), Some(end))
        }
        "sprint" | "last_sprint" => match sprint_config(conn)? {
            Some(config) => {
                let current = config.sprint_of(today);
                let number = if scope == "sprint" { current } else { current - 1 };
                let (start, end) = config.range_of(number);
                sprint_number = Some(number);
                (Some(start), Some(end))
            }
            None => (None, None),
        },
        _ => (None, None),
    };

    Ok(ResolvedRange {
        scope: scope.to_string(),
        sprint_number,
        from,
        to,
    })
}

pub fn hours_report(
    conn: &Connection,
    user_id: Option<i64>,
    scope: &str,
    custom_from: &str,
    custom_to: &str,
) -> Result<HoursReport> {
    let today = Local::now().date_naive();
    let resolved = resolve_range(conn, scope, today)?;
    let (from, to) = match (resolved.from, resolved.to) {
        (Some(from), Some(to)) => (from, to),
        _ => match (
            calendar::parse_date(custom_from.trim()),
            calendar::parse_date(custom_to.trim()),
        ) {
            (Some(from), Some(to)) => (from, to),
            _ => {
                let (start, end) = calendar::week_range(today);
                (start, end)
            }
        },
    };

    let Some(user_id) = user_id else {
        return Ok(HoursReport {
            scope: resolved.scope,
            sprint_number: resolved.sprint_number,
            date_from: calendar::format_date(from),
            date_to: calendar::format_date(to),
            required: 0.0,
            filled: 0.0,
            diff: 0.0,
            has_user: false,
        });
    };

    let daily_required: f64 = conn
        .query_row(
            "SELECT daily_required_hours FROM users WHERE id = ?1",
            params![user_id],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(8.0);

    let from_text = calendar::format_date(from);
    let to_text = calendar::format_date(to);

    let mut stmt =
        conn.prepare("SELECT date, is_workday FROM holidays WHERE date BETWEEN ?1 AND ?2")?;
    let mut calendar = calendar::Calendar::default();
    for row in stmt.query_map(params![from_text, to_text], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? != 0))
    })? {
        let Ok((value, is_workday)) = row else {
            continue;
        };
        let Some(date) = calendar::parse_date(&value) else {
            continue;
        };
        if is_workday {
            calendar.extra_workdays.insert(date);
        } else {
            calendar.holidays.insert(date);
        }
    }

    let mut stmt = conn.prepare(
        "SELECT leave_date, SUM(hours) FROM user_leaves
         WHERE user_id = ?1 AND leave_date BETWEEN ?2 AND ?3 GROUP BY leave_date",
    )?;
    let leaves: HashMap<NaiveDate, f64> = stmt
        .query_map(params![user_id, from_text, to_text], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?
        .filter_map(|row| row.ok())
        .filter_map(|(date, hours)| calendar::parse_date(&date).map(|date| (date, hours)))
        .collect();

    let mut stmt = conn.prepare(
        "SELECT work_date, SUM(hours) FROM entries
         WHERE user_id = ?1 AND work_date BETWEEN ?2 AND ?3 GROUP BY work_date",
    )?;
    let filled: HashMap<NaiveDate, f64> = stmt
        .query_map(params![user_id, from_text, to_text], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?
        .filter_map(|row| row.ok())
        .filter_map(|(date, hours)| calendar::parse_date(&date).map(|date| (date, hours)))
        .collect();

    let summary = calendar::summarize(from, to, daily_required, &calendar, &leaves, &filled);

    Ok(HoursReport {
        scope: resolved.scope,
        sprint_number: resolved.sprint_number,
        date_from: from_text,
        date_to: to_text,
        required: summary.required,
        filled: summary.filled,
        diff: summary.diff,
        has_user: true,
    })
}

fn now() -> String {
    Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

#[allow(dead_code)]
fn day_after(date: NaiveDate) -> NaiveDate {
    date + Duration::days(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup() -> Connection {
        db::open_in_memory().unwrap()
    }

    fn status_id(conn: &Connection, name: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM statuses WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn sample_entry(user_id: Option<i64>, title: &str, work_date: &str, hours: f64) -> EntryInput {
        EntryInput {
            id: None,
            user_id,
            title: title.to_string(),
            content: String::new(),
            ticket: String::new(),
            status_id: None,
            work_date: work_date.to_string(),
            hours,
        }
    }

    #[test]
    fn new_entry_gets_the_default_status() {
        let conn = setup();
        let id = save_entry(&conn, &sample_entry(None, "第一筆", "2026-09-18", 1.0)).unwrap();
        let entry = get_entry(&conn, id).unwrap().unwrap();
        assert_eq!(entry.status_name, "new");
    }

    #[test]
    fn users_are_listed_oldest_first() {
        let conn = setup();
        let first = create_user(&conn, "最早建立", 8.0).unwrap();
        let second = create_user(&conn, "接著建立", 8.0).unwrap();
        let third = create_user(&conn, "最後建立", 8.0).unwrap();

        let ids: Vec<i64> = list_users(&conn).unwrap().iter().map(|user| user.id).collect();
        assert_eq!(ids, vec![first, second, third]);
    }

    #[test]
    fn deleting_a_user_keeps_entries_and_clears_the_owner() {
        let conn = setup();
        let user_id = create_user(&conn, "amanda", 8.0).unwrap();
        let entry_id =
            save_entry(&conn, &sample_entry(Some(user_id), "有主的紀錄", "2026-09-18", 3.0)).unwrap();

        delete_user(&conn, user_id).unwrap();

        let entry = get_entry(&conn, entry_id).unwrap().unwrap();
        assert_eq!(entry.user_id, None);
        assert_eq!(entry.title, "有主的紀錄");
    }

    #[test]
    fn deleting_a_user_removes_their_leaves() {
        let conn = setup();
        let user_id = create_user(&conn, "amanda", 8.0).unwrap();
        save_leave(&conn, None, user_id, "2026-09-18", 4.0, "半天").unwrap();

        delete_user(&conn, user_id).unwrap();

        assert!(list_leaves(&conn, None).unwrap().is_empty());
    }

    #[test]
    fn deleting_a_status_moves_entries_to_the_replacement() {
        let conn = setup();
        let active = status_id(&conn, "active");
        let closed = status_id(&conn, "closed");
        let mut input = sample_entry(None, "進行中", "2026-09-18", 2.0);
        input.status_id = Some(active);
        let entry_id = save_entry(&conn, &input).unwrap();

        delete_status(&conn, active, closed).unwrap();

        let entry = get_entry(&conn, entry_id).unwrap().unwrap();
        assert_eq!(entry.status_id, closed);
        assert!(list_statuses(&conn)
            .unwrap()
            .iter()
            .all(|status| status.name != "active"));
    }

    #[test]
    fn the_default_status_cannot_be_deleted() {
        let conn = setup();
        let new_id = status_id(&conn, "new");
        let closed = status_id(&conn, "closed");
        assert!(delete_status(&conn, new_id, closed).is_err());
    }

    #[test]
    fn unowned_filter_finds_entries_without_an_owner() {
        let conn = setup();
        let user_id = create_user(&conn, "amanda", 8.0).unwrap();
        save_entry(&conn, &sample_entry(Some(user_id), "有主", "2026-09-18", 1.0)).unwrap();
        save_entry(&conn, &sample_entry(None, "無主", "2026-09-18", 1.0)).unwrap();

        let filter = EntryQuery {
            user_filter: UserFilter::Unowned,
            ..EntryQuery::default()
        };
        let found = search_entries(&conn, &filter).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "無主");
    }

    #[test]
    fn keyword_search_covers_title_content_and_ticket() {
        let conn = setup();
        let mut with_content = sample_entry(None, "標題甲", "2026-09-18", 1.0);
        with_content.content = "內容提到登入頁".to_string();
        let mut with_ticket = sample_entry(None, "標題乙", "2026-09-18", 1.0);
        with_ticket.ticket = "登入頁-123".to_string();
        save_entry(&conn, &sample_entry(None, "登入頁改版", "2026-09-18", 1.0)).unwrap();
        save_entry(&conn, &with_content).unwrap();
        save_entry(&conn, &with_ticket).unwrap();
        save_entry(&conn, &sample_entry(None, "不相關", "2026-09-18", 1.0)).unwrap();

        let filter = EntryQuery {
            keyword: "登入頁".to_string(),
            ..EntryQuery::default()
        };
        assert_eq!(search_entries(&conn, &filter).unwrap().len(), 3);
    }

    #[test]
    fn linking_puts_both_entries_in_one_group() {
        let conn = setup();
        let bug = save_entry(&conn, &sample_entry(None, "修登入頁 bug", "2026-09-18", 1.0)).unwrap();
        let revamp = save_entry(&conn, &sample_entry(None, "登入頁改版", "2026-09-17", 1.0)).unwrap();

        link_entries(&conn, bug, revamp).unwrap();

        let from_bug = entry_detail(&conn, bug).unwrap().unwrap();
        assert_eq!(from_bug.related.len(), 1);
        assert_eq!(from_bug.related[0].title, "登入頁改版");

        let from_revamp = entry_detail(&conn, revamp).unwrap().unwrap();
        assert_eq!(from_revamp.related.len(), 1);
        assert_eq!(from_revamp.related[0].title, "修登入頁 bug");
    }

    #[test]
    fn a_third_entry_joins_the_existing_group() {
        let conn = setup();
        let first = save_entry(&conn, &sample_entry(None, "甲", "2026-09-18", 1.0)).unwrap();
        let second = save_entry(&conn, &sample_entry(None, "乙", "2026-09-17", 1.0)).unwrap();
        let third = save_entry(&conn, &sample_entry(None, "丙", "2026-09-16", 1.0)).unwrap();

        link_entries(&conn, first, second).unwrap();
        link_entries(&conn, second, third).unwrap();

        for id in [first, second, third] {
            let detail = entry_detail(&conn, id).unwrap().unwrap();
            assert_eq!(detail.related.len(), 2, "紀錄 {id} 應該看到另外兩筆");
        }
    }

    #[test]
    fn linking_two_groups_merges_them() {
        let conn = setup();
        let a1 = save_entry(&conn, &sample_entry(None, "A1", "2026-09-18", 1.0)).unwrap();
        let a2 = save_entry(&conn, &sample_entry(None, "A2", "2026-09-17", 1.0)).unwrap();
        let b1 = save_entry(&conn, &sample_entry(None, "B1", "2026-09-16", 1.0)).unwrap();
        let b2 = save_entry(&conn, &sample_entry(None, "B2", "2026-09-15", 1.0)).unwrap();

        link_entries(&conn, a1, a2).unwrap();
        link_entries(&conn, b1, b2).unwrap();
        link_entries(&conn, a1, b1).unwrap();

        for id in [a1, a2, b1, b2] {
            let detail = entry_detail(&conn, id).unwrap().unwrap();
            assert_eq!(detail.related.len(), 3);
        }
    }

    #[test]
    fn unlinking_removes_only_that_entry_from_the_group() {
        let conn = setup();
        let first = save_entry(&conn, &sample_entry(None, "甲", "2026-09-18", 1.0)).unwrap();
        let second = save_entry(&conn, &sample_entry(None, "乙", "2026-09-17", 1.0)).unwrap();
        let third = save_entry(&conn, &sample_entry(None, "丙", "2026-09-16", 1.0)).unwrap();
        link_entries(&conn, first, second).unwrap();
        link_entries(&conn, second, third).unwrap();

        unlink_entry(&conn, second).unwrap();

        assert!(entry_detail(&conn, second).unwrap().unwrap().related.is_empty());
        let remaining = entry_detail(&conn, first).unwrap().unwrap();
        assert_eq!(remaining.related.len(), 1);
        assert_eq!(remaining.related[0].title, "丙");
    }

    #[test]
    fn a_group_left_with_one_member_dissolves() {
        let conn = setup();
        let first = save_entry(&conn, &sample_entry(None, "甲", "2026-09-18", 1.0)).unwrap();
        let second = save_entry(&conn, &sample_entry(None, "乙", "2026-09-17", 1.0)).unwrap();
        link_entries(&conn, first, second).unwrap();

        unlink_entry(&conn, second).unwrap();

        let group: Option<i64> = conn
            .query_row(
                "SELECT group_id FROM entries WHERE id = ?1",
                params![first],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(group, None);
    }

    #[test]
    fn deleting_an_entry_dissolves_a_two_member_group() {
        let conn = setup();
        let first = save_entry(&conn, &sample_entry(None, "甲", "2026-09-18", 1.0)).unwrap();
        let second = save_entry(&conn, &sample_entry(None, "乙", "2026-09-17", 1.0)).unwrap();
        link_entries(&conn, first, second).unwrap();

        delete_entry(&conn, second).unwrap();

        assert!(entry_detail(&conn, first).unwrap().unwrap().related.is_empty());
    }

    #[test]
    fn link_candidates_exclude_self_and_group_members() {
        let conn = setup();
        let bug = save_entry(&conn, &sample_entry(None, "修登入頁 bug", "2026-09-18", 1.0)).unwrap();
        let revamp = save_entry(&conn, &sample_entry(None, "登入頁改版", "2026-09-17", 1.0)).unwrap();
        let other = save_entry(&conn, &sample_entry(None, "報表匯出", "2026-09-16", 1.0)).unwrap();
        link_entries(&conn, bug, revamp).unwrap();

        let ids: Vec<i64> = link_candidates(&conn, bug, "")
            .unwrap()
            .iter()
            .map(|item| item.id)
            .collect();
        assert_eq!(ids, vec![other]);
    }

    #[test]
    fn link_candidates_are_ordered_by_most_recently_updated() {
        let conn = setup();
        let anchor = save_entry(&conn, &sample_entry(None, "主紀錄", "2026-09-18", 1.0)).unwrap();
        let first = save_entry(&conn, &sample_entry(None, "較早更新", "2026-09-17", 1.0)).unwrap();
        let second = save_entry(&conn, &sample_entry(None, "較晚更新", "2026-09-16", 1.0)).unwrap();
        conn.execute(
            "UPDATE entries SET updated_at = '2026-09-01T09:00:00' WHERE id = ?1",
            params![first],
        )
        .unwrap();
        conn.execute(
            "UPDATE entries SET updated_at = '2026-09-10T09:00:00' WHERE id = ?1",
            params![second],
        )
        .unwrap();

        let ids: Vec<i64> = link_candidates(&conn, anchor, "")
            .unwrap()
            .iter()
            .map(|item| item.id)
            .collect();
        assert_eq!(ids, vec![second, first]);
    }

    #[test]
    fn hours_report_without_a_user_reports_no_user() {
        let conn = setup();
        let report = hours_report(&conn, None, "week", "", "").unwrap();
        assert!(!report.has_user);
        assert_eq!(report.required, 0.0);
    }

    #[test]
    fn hours_report_counts_every_status() {
        let conn = setup();
        let user_id = create_user(&conn, "amanda", 8.0).unwrap();
        let closed = status_id(&conn, "closed");
        let mut done = sample_entry(Some(user_id), "已結案", "2026-09-14", 8.0);
        done.status_id = Some(closed);
        save_entry(&conn, &done).unwrap();

        let report = hours_report(&conn, Some(user_id), "custom", "2026-09-14", "2026-09-14").unwrap();
        assert_eq!(report.filled, 8.0);
        assert_eq!(report.required, 8.0);
        assert_eq!(report.diff, 0.0);
    }

    #[test]
    fn hours_report_subtracts_leave_from_required() {
        let conn = setup();
        let user_id = create_user(&conn, "amanda", 8.0).unwrap();
        save_leave(&conn, None, user_id, "2026-09-14", 4.0, "半天假").unwrap();
        save_entry(&conn, &sample_entry(Some(user_id), "半天工作", "2026-09-14", 4.0)).unwrap();

        let report = hours_report(&conn, Some(user_id), "custom", "2026-09-14", "2026-09-14").unwrap();
        assert_eq!(report.required, 4.0);
        assert_eq!(report.diff, 0.0);
    }

    #[test]
    fn hours_report_ignores_holidays_in_required() {
        let conn = setup();
        let user_id = create_user(&conn, "amanda", 8.0).unwrap();
        save_holiday(&conn, "2026-09-14", "測試假日", false).unwrap();

        let report = hours_report(&conn, Some(user_id), "custom", "2026-09-14", "2026-09-14").unwrap();
        assert_eq!(report.required, 0.0);
    }

    #[test]
    fn resolve_range_falls_back_when_sprint_is_not_configured() {
        let conn = setup();
        let today = calendar::parse_date("2026-09-18").unwrap();
        let resolved = resolve_range(&conn, "sprint", today).unwrap();
        assert!(resolved.from.is_none());
        assert!(resolved.sprint_number.is_none());
    }

    #[test]
    fn resolve_range_uses_the_configured_sprint_anchor() {
        let conn = setup();
        set_setting(&conn, "sprint_anchor_number", "12").unwrap();
        set_setting(&conn, "sprint_anchor_start_date", "2026-09-01").unwrap();
        set_setting(&conn, "sprint_length_days", "14").unwrap();

        let today = calendar::parse_date("2026-09-18").unwrap();
        let resolved = resolve_range(&conn, "sprint", today).unwrap();
        assert_eq!(resolved.sprint_number, Some(13));
        assert_eq!(resolved.from, calendar::parse_date("2026-09-15"));
        assert_eq!(resolved.to, calendar::parse_date("2026-09-28"));

        let previous = resolve_range(&conn, "last_sprint", today).unwrap();
        assert_eq!(previous.sprint_number, Some(12));
        assert_eq!(previous.from, calendar::parse_date("2026-09-01"));
    }
}
