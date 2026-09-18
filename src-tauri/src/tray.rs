use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub const TRAY_ID: &str = "worklog-tray";

const SIZE: usize = 32;
const ACCENT: [u8; 3] = [0x25, 0x63, 0xeb];

fn icon() -> Image<'static> {
    let mut data = vec![0u8; SIZE * SIZE * 4];
    let centre = (SIZE as f32 - 1.0) / 2.0;
    let radius = 15.0;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - centre;
            let dy = y as f32 - centre;
            let distance = (dx * dx + dy * dy).sqrt();
            let alpha = if distance <= radius - 1.0 {
                255.0
            } else if distance <= radius {
                (radius - distance) * 255.0
            } else {
                0.0
            };
            if alpha > 0.0 {
                let index = (y * SIZE + x) * 4;
                data[index] = ACCENT[0];
                data[index + 1] = ACCENT[1];
                data[index + 2] = ACCENT[2];
                data[index + 3] = alpha as u8;
            }
        }
    }

    for y in 9..17 {
        paint_white(&mut data, 15, y);
        paint_white(&mut data, 16, y);
    }
    for x in 16..23 {
        paint_white(&mut data, x, 15);
        paint_white(&mut data, x, 16);
    }

    Image::new_owned(data, SIZE as u32, SIZE as u32)
}

fn paint_white(data: &mut [u8], x: usize, y: usize) {
    let index = (y * SIZE + x) * 4;
    data[index] = 0xff;
    data[index + 1] = 0xff;
    data[index + 2] = 0xff;
    data[index + 3] = 0xff;
}

pub fn refresh(
    app: &AppHandle,
    tooltip: &str,
    show_label: &str,
    quit_label: &str,
) -> tauri::Result<()> {
    app.remove_tray_by_id(TRAY_ID);

    let show = MenuItem::with_id(app, "show", show_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon())
        .tooltip(tooltip)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => reveal(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                reveal(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn reveal(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
