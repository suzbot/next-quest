use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

use crate::commands::{AppTrayState, DbState};
use crate::db;

const TRAY_ID: &str = "nq_tray";

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_menu(app)?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id().as_ref());
        })
        .build(app)?;

    Ok(())
}

pub fn rebuild_tray_menu(app: &AppHandle) {
    let tray_state = app.state::<AppTrayState>();
    let tray = tray_state.0.lock().unwrap();

    if let Ok(menu) = build_menu_from_state(app, &tray) {
        if let Some(tray_icon) = app.tray_by_id(TRAY_ID) {
            let _ = tray_icon.set_menu(Some(menu));
        }
    }
}

fn build_menu(app: &tauri::App) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let tray_state = app.state::<AppTrayState>();
    let tray = tray_state.0.lock().map_err(|e| e.to_string())?;
    build_menu_from_state(app, &tray)
}

fn build_menu_from_state<M: Manager<tauri::Wry>>(
    manager: &M,
    tray: &crate::commands::TrayStateInner,
) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let menu = Menu::new(manager)?;

    let cta_label = if tray.call_to_adventure {
        "Call to Adventure: ON"
    } else {
        "Call to Adventure: OFF"
    };
    let cta_toggle = MenuItem::with_id(manager, "call_to_adventure", cta_label, true, None::<&str>)?;
    menu.append(&cta_toggle)?;

    menu.append(&PredefinedMenuItem::separator(manager)?)?;

    let open = MenuItem::with_id(manager, "open_app", "Open Next Quest", true, None::<&str>)?;
    let quit = MenuItem::with_id(manager, "quit", "Quit", true, None::<&str>)?;
    menu.append(&open)?;
    menu.append(&quit)?;

    Ok(menu)
}

fn handle_menu_event(app: &AppHandle, event_id: &str) {
    match event_id {
        "call_to_adventure" => {
            let tray_state = app.state::<AppTrayState>();
            let mut tray = tray_state.0.lock().unwrap();
            tray.call_to_adventure = !tray.call_to_adventure;
            if tray.call_to_adventure {
                tray.reset_fire_time();
            }
            let (enabled, interval_mins) = (tray.call_to_adventure, tray.cta_interval_secs / 60);
            drop(tray);

            // Persist to DB
            let db_state = app.state::<DbState>();
            if let Ok(conn) = db_state.0.lock() {
                let _ = db::set_settings_db(&conn, enabled, interval_mins);
            }

            rebuild_tray_menu(app);
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("settings-changed", ());
            }
        }

        "open_app" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }

        "quit" => {
            app.exit(0);
        }

        _ => {}
    }
}
