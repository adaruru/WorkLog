use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Local;
use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::calendar;
use crate::models::*;
use crate::store;
use crate::tray;

pub struct AppState {
    pub conn: Mutex<Connection>,
}

type Answer<T> = std::result::Result<T, String>;

fn with_conn<T, F>(state: &State<AppState>, action: F) -> Answer<T>
where
    F: FnOnce(&Connection) -> rusqlite::Result<T>,
{
    let conn = state.conn.lock().map_err(|_| "資料庫連線被佔用".to_string())?;
    action(&conn).map_err(|error| error.to_string())
}

#[derive(Serialize)]
pub struct Bootstrap {
    pub settings: HashMap<String, String>,
    pub users: Vec<User>,
    pub statuses: Vec<Status>,
    pub db_path: String,
}

#[derive(Serialize)]
pub struct RangeDto {
    pub scope: String,
    pub sprint_number: Option<i64>,
    pub date_from: String,
    pub date_to: String,
}

#[tauri::command]
pub fn log_front(level: String, message: String) {
    eprintln!("[front:{level}] {message}");
}

#[tauri::command]
pub fn bootstrap(state: State<AppState>) -> Answer<Bootstrap> {
    with_conn(&state, |conn| {
        Ok(Bootstrap {
            settings: store::all_settings(conn)?,
            users: store::list_users(conn)?,
            statuses: store::list_statuses(conn)?,
            db_path: store::get_setting(conn, "db_path")?
                .unwrap_or_else(|| crate::db::default_db_path().to_string_lossy().to_string()),
        })
    })
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Answer<HashMap<String, String>> {
    with_conn(&state, store::all_settings)
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Answer<()> {
    with_conn(&state, |conn| store::set_setting(conn, &key, &value))
}

#[tauri::command]
pub fn list_users(state: State<AppState>) -> Answer<Vec<User>> {
    with_conn(&state, store::list_users)
}

#[tauri::command]
pub fn create_user(state: State<AppState>, name: String, daily_required_hours: f64) -> Answer<i64> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("使用者名稱不可空白".to_string());
    }
    with_conn(&state, |conn| {
        store::create_user(conn, &name, daily_required_hours)
    })
}

#[tauri::command]
pub fn update_user(
    state: State<AppState>,
    id: i64,
    name: String,
    daily_required_hours: f64,
    is_active: bool,
) -> Answer<()> {
    with_conn(&state, |conn| {
        store::update_user(conn, id, name.trim(), daily_required_hours, is_active)
    })
}

#[tauri::command]
pub fn delete_user(state: State<AppState>, id: i64) -> Answer<()> {
    with_conn(&state, |conn| store::delete_user(conn, id))
}

#[tauri::command]
pub fn list_statuses(state: State<AppState>) -> Answer<Vec<Status>> {
    with_conn(&state, store::list_statuses)
}

#[tauri::command]
pub fn create_status(state: State<AppState>, name: String, color: String) -> Answer<i64> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("狀態名稱不可空白".to_string());
    }
    with_conn(&state, |conn| store::create_status(conn, &name, &color))
}

#[tauri::command]
pub fn update_status(state: State<AppState>, id: i64, name: String, color: String) -> Answer<()> {
    with_conn(&state, |conn| {
        store::update_status(conn, id, name.trim(), &color)
    })
}

#[tauri::command]
pub fn delete_status(state: State<AppState>, id: i64, replacement_id: i64) -> Answer<()> {
    with_conn(&state, |conn| {
        store::delete_status(conn, id, replacement_id)
    })
}

#[tauri::command]
pub fn search_entries(state: State<AppState>, query: EntryQuery) -> Answer<Vec<Entry>> {
    with_conn(&state, |conn| store::search_entries(conn, &query))
}

#[tauri::command]
pub fn entry_detail(state: State<AppState>, id: i64) -> Answer<Option<EntryDetail>> {
    with_conn(&state, |conn| store::entry_detail(conn, id))
}

#[tauri::command]
pub fn save_entry(state: State<AppState>, input: EntryInput) -> Answer<i64> {
    if input.title.trim().is_empty() {
        return Err("標題不可空白".to_string());
    }
    if calendar::parse_date(input.work_date.trim()).is_none() {
        return Err("工作日期格式需為 yyyy-MM-dd".to_string());
    }
    with_conn(&state, |conn| store::save_entry(conn, &input))
}

#[tauri::command]
pub fn set_entry_status(state: State<AppState>, id: i64, status_id: i64) -> Answer<()> {
    with_conn(&state, |conn| store::set_entry_status(conn, id, status_id))
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, id: i64) -> Answer<()> {
    with_conn(&state, |conn| store::delete_entry(conn, id))
}

#[tauri::command]
pub fn link_candidates(
    state: State<AppState>,
    entry_id: i64,
    keyword: String,
) -> Answer<Vec<EntryRef>> {
    with_conn(&state, |conn| {
        store::link_candidates(conn, entry_id, &keyword)
    })
}

#[tauri::command]
pub fn link_entries(state: State<AppState>, entry_id: i64, linked_entry_id: i64) -> Answer<()> {
    with_conn(&state, |conn| {
        store::link_entries(conn, entry_id, linked_entry_id)
    })
}

#[tauri::command]
pub fn unlink_entry(state: State<AppState>, id: i64) -> Answer<()> {
    with_conn(&state, |conn| store::unlink_entry(conn, id))
}

#[tauri::command]
pub fn list_holidays(state: State<AppState>) -> Answer<Vec<Holiday>> {
    with_conn(&state, store::list_holidays)
}

#[tauri::command]
pub fn save_holiday(
    state: State<AppState>,
    date: String,
    name: String,
    is_workday: bool,
) -> Answer<()> {
    if calendar::parse_date(date.trim()).is_none() {
        return Err("假日日期格式需為 yyyy-MM-dd".to_string());
    }
    with_conn(&state, |conn| {
        store::save_holiday(conn, date.trim(), name.trim(), is_workday)
    })
}

#[tauri::command]
pub fn import_holidays(
    state: State<AppState>,
    year: i32,
    items: Vec<Holiday>,
    replace: bool,
) -> Answer<usize> {
    with_conn(&state, |conn| {
        if replace {
            store::clear_holidays_in_year(conn, year)?;
        }
        store::import_holidays(conn, &items)
    })
}

#[tauri::command]
pub fn delete_holiday(state: State<AppState>, date: String) -> Answer<()> {
    with_conn(&state, |conn| store::delete_holiday(conn, &date))
}

#[tauri::command]
pub fn list_leaves(state: State<AppState>, user_id: Option<i64>) -> Answer<Vec<Leave>> {
    with_conn(&state, |conn| store::list_leaves(conn, user_id))
}

#[tauri::command]
pub fn save_leave(
    state: State<AppState>,
    id: Option<i64>,
    user_id: i64,
    leave_date: String,
    hours: f64,
    note: String,
) -> Answer<()> {
    if calendar::parse_date(leave_date.trim()).is_none() {
        return Err("請假日期格式需為 yyyy-MM-dd".to_string());
    }
    with_conn(&state, |conn| {
        store::save_leave(conn, id, user_id, leave_date.trim(), hours, note.trim())
    })
}

#[tauri::command]
pub fn delete_leave(state: State<AppState>, id: i64) -> Answer<()> {
    with_conn(&state, |conn| store::delete_leave(conn, id))
}

#[tauri::command]
pub fn sprint_settings(state: State<AppState>) -> Answer<SprintSettings> {
    with_conn(&state, store::sprint_settings)
}

#[tauri::command]
pub fn resolve_range(state: State<AppState>, scope: String) -> Answer<RangeDto> {
    let today = Local::now().date_naive();
    with_conn(&state, |conn| {
        let resolved = store::resolve_range(conn, &scope, today)?;
        Ok(RangeDto {
            scope: resolved.scope,
            sprint_number: resolved.sprint_number,
            date_from: resolved.from.map(calendar::format_date).unwrap_or_default(),
            date_to: resolved.to.map(calendar::format_date).unwrap_or_default(),
        })
    })
}

#[tauri::command]
pub fn hours_report(
    state: State<AppState>,
    user_id: Option<i64>,
    scope: String,
    date_from: String,
    date_to: String,
) -> Answer<HoursReport> {
    with_conn(&state, |conn| {
        store::hours_report(conn, user_id, &scope, &date_from, &date_to)
    })
}

fn native_theme(value: &str) -> tauri::Theme {
    match value {
        "light" => tauri::Theme::Light,
        _ => tauri::Theme::Dark,
    }
}

#[tauri::command]
pub async fn open_settings_window(app: AppHandle, theme: String) -> Answer<()> {
    let native = native_theme(&theme);

    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.set_theme(Some(native));
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        &app,
        "settings",
        tauri::WebviewUrl::App("settings.html".into()),
    )
    .title("WorkLog")
    .inner_size(560.0, 460.0)
    .min_inner_size(460.0, 380.0)
    .theme(Some(native))
    .build()
    .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_window_theme(app: AppHandle, theme: String) -> Answer<()> {
    let native = native_theme(&theme);
    for window in app.webview_windows().values() {
        let _ = window.set_theme(Some(native));
    }
    Ok(())
}

#[tauri::command]
pub fn update_tray(
    app: AppHandle,
    tooltip: String,
    show_label: String,
    quit_label: String,
) -> Answer<()> {
    tray::refresh(&app, &tooltip, &show_label, &quit_label).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn current_db_path(state: State<AppState>) -> Answer<String> {
    with_conn(&state, |conn| {
        Ok(store::get_setting(conn, "db_path")?
            .unwrap_or_else(|| crate::db::default_db_path().to_string_lossy().to_string()))
    })
}

#[tauri::command]
pub fn change_db_path(state: State<AppState>, path: String) -> Answer<()> {
    let path = path.trim().to_string();
    if path.is_empty() {
        return Err("資料庫路徑不可空白".to_string());
    }
    let opened = crate::db::open(std::path::Path::new(&path)).map_err(|error| error.to_string())?;
    store::set_setting(&opened, "db_path", &path).map_err(|error| error.to_string())?;

    let mut guard = state.conn.lock().map_err(|_| "資料庫連線被佔用".to_string())?;
    *guard = opened;
    Ok(())
}

#[tauri::command]
pub fn reveal_db_folder(state: State<AppState>) -> Answer<()> {
    let path = with_conn(&state, |conn| {
        Ok(store::get_setting(conn, "db_path")?
            .unwrap_or_else(|| crate::db::default_db_path().to_string_lossy().to_string()))
    })?;
    let folder = std::path::Path::new(&path)
        .parent()
        .map(|dir| dir.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    std::process::Command::new("explorer")
        .arg(folder)
        .spawn()
        .map_err(|error| error.to_string())?;
    Ok(())
}
