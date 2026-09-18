#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod calendar;
mod commands;
mod db;
mod models;
mod query;
mod store;
mod tray;

use std::path::PathBuf;
use std::sync::Mutex;

use commands::AppState;
use tauri::{LogicalSize, Manager, WindowEvent};

fn open_configured_db() -> rusqlite::Result<rusqlite::Connection> {
    let fallback = db::default_db_path();
    let conn = db::open(&fallback)?;
    let configured = store::get_setting(&conn, "db_path")?
        .map(PathBuf::from)
        .filter(|path| path != &fallback);

    match configured {
        Some(path) => db::open(&path).or(Ok(conn)),
        None => Ok(conn),
    }
}

fn stored_window_size(conn: &rusqlite::Connection) -> Option<(f64, f64)> {
    let raw = store::get_setting(conn, "window_layout").ok()??;
    let parsed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let width = parsed.get("width")?.as_f64()?;
    let height = parsed.get("height")?.as_f64()?;
    (width >= 680.0 && height >= 420.0).then_some((width, height))
}

fn remember_window_size(window: &tauri::Window) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let Ok(size) = window.inner_size() else {
        return;
    };
    let logical = size.to_logical::<f64>(scale);

    let state = window.app_handle().state::<AppState>();
    let Ok(conn) = state.conn.lock() else {
        return;
    };
    let stored = store::get_setting(&conn, "window_layout")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    let list_width = stored
        .as_ref()
        .and_then(|value| value.get("listWidth"))
        .and_then(|value| value.as_f64())
        .unwrap_or(264.0);

    let payload = serde_json::json!({
        "width": logical.width,
        "height": logical.height,
        "listWidth": list_width
    });
    let _ = store::set_setting(&conn, "window_layout", &payload.to_string());
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let conn = open_configured_db()?;
            let size = stored_window_size(&conn);
            app.manage(AppState {
                conn: Mutex::new(conn),
            });

            if let (Some((width, height)), Some(window)) = (size, app.get_webview_window("main")) {
                let _ = window.set_size(LogicalSize::new(width, height));
            }

            tray::refresh(app.handle(), "WorkLog", "顯示視窗", "離開")?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    remember_window_size(window);
                    if cfg!(debug_assertions) {
                        return;
                    }
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::log_front,
            commands::bootstrap,
            commands::get_settings,
            commands::set_setting,
            commands::list_users,
            commands::create_user,
            commands::update_user,
            commands::delete_user,
            commands::list_statuses,
            commands::create_status,
            commands::update_status,
            commands::delete_status,
            commands::search_entries,
            commands::entry_detail,
            commands::save_entry,
            commands::set_entry_status,
            commands::delete_entry,
            commands::link_candidates,
            commands::link_entries,
            commands::unlink_entries,
            commands::list_holidays,
            commands::save_holiday,
            commands::delete_holiday,
            commands::list_leaves,
            commands::save_leave,
            commands::delete_leave,
            commands::sprint_settings,
            commands::resolve_range,
            commands::hours_report,
            commands::open_settings_window,
            commands::update_tray,
            commands::current_db_path,
            commands::change_db_path,
            commands::reveal_db_folder,
        ])
        .run(tauri::generate_context!())
        .expect("WorkLog 啟動失敗");
}
