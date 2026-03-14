#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod tray;

use commands::{AppTimerState, AppTrayState, DbState, TimerStateInner, TrayStateInner};
use std::sync::Mutex;
use tauri::{Manager, WindowEvent};

fn main() {
    // Store the database in the app's data directory
    let db_path = dirs::data_dir()
        .expect("Could not find data directory")
        .join("com.nextquest.desktop")
        .join("next-quest.db");

    // Ensure the directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create data directory");
    }

    let conn = db::init_db(&db_path);

    tauri::Builder::default()
        .manage(DbState(Mutex::new(conn)))
        .manage(AppTimerState(Mutex::new(TimerStateInner::default())))
        .manage(AppTrayState(Mutex::new(TrayStateInner::new())))
        .setup(|app| {
            tray::setup_tray(app).expect("Failed to set up system tray");

            // Spawn Call to Adventure polling thread
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                cta_poll_loop(app_handle);
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_quests,
            commands::get_completions,
            commands::add_quest,
            commands::complete_quest,
            commands::update_quest,
            commands::delete_quest,
            commands::delete_completion,
            commands::reorder_quests,
            commands::get_character,
            commands::update_character,
            commands::get_attributes,
            commands::get_skills,
            commands::get_quest_links,
            commands::set_quest_links,
            commands::reset_character,
            commands::reset_quests,
            commands::reset_completions,
            commands::get_next_quest,
            commands::start_timer,
            commands::cancel_timer,
            commands::complete_timer,
            commands::get_timer_state,
            commands::get_settings,
            commands::set_cta_interval,
            commands::toggle_call_to_adventure,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Next Quest");
}

fn cta_poll_loop(app: tauri::AppHandle) {
    loop {
        // Read the current interval
        let interval_secs = {
            let tray_state = app.state::<AppTrayState>();
            let tray = tray_state.0.lock().unwrap();
            tray.cta_interval_secs
        };

        // Sleep for the interval
        std::thread::sleep(std::time::Duration::from_secs(interval_secs));

        // Check conditions
        let should_fire = {
            let tray_state = app.state::<AppTrayState>();
            let tray = tray_state.0.lock().unwrap();
            if !tray.call_to_adventure {
                false
            } else {
                let timer_state = app.state::<AppTimerState>();
                let timer = timer_state.0.lock().unwrap();
                if timer.quest_id.is_some() {
                    // Timer running — don't interrupt
                    false
                } else {
                    // Check if main window is focused
                    let window_focused = app
                        .get_webview_window("main")
                        .map(|w| w.is_focused().unwrap_or(false))
                        .unwrap_or(false);
                    !window_focused
                }
            }
        };

        if should_fire {
            // Check if there's actually a quest to suggest
            let has_quest = {
                let db_state = app.state::<DbState>();
                let conn = db_state.0.lock().unwrap();
                db::get_next_quest(&conn, 0)
                    .map(|q| q.is_some())
                    .unwrap_or(false)
            };

            if has_quest {
                // For now, emit an event. Step 5b will show the overlay window.
                let _ = tauri::Emitter::emit(&app, "call-to-adventure", ());
            }
        }
    }
}
