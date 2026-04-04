#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod tray;

use nq_core::db;

use commands::{AppSkipState, AppTimerState, AppTrayState, DbState, SkipStateInner, TimerStateInner, TrayStateInner};
use std::sync::Mutex;
use tauri::{Emitter, Manager, WindowEvent};

fn main() {
    let db_path = nq_core::db_path();
    let conn = db::init_db(&db_path);

    // Load persisted settings
    let (cta_enabled, cta_interval_minutes, _debug_scoring) = db::get_settings_db(&conn).unwrap_or((false, 20, false));
    let mut tray_state = TrayStateInner::new();
    tray_state.call_to_adventure = cta_enabled;
    tray_state.cta_interval_secs = cta_interval_minutes * 60;
    if cta_enabled {
        tray_state.reset_fire_time();
    }

    tauri::Builder::default()
        .manage(DbState(Mutex::new(conn)))
        .manage(AppTimerState(Mutex::new(TimerStateInner::default())))
        .manage(AppTrayState(Mutex::new(tray_state)))
        .manage(AppSkipState(Mutex::new(SkipStateInner::default())))
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
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_quests,
            commands::get_quest_list,
            commands::get_sagas,
            commands::get_sagas_with_progress,
            commands::check_saga_completion,
            commands::check_saga_completion_for_quest,
            commands::add_saga,
            commands::update_saga,
            commands::delete_saga,
            commands::get_saga_steps,
            commands::add_saga_step,
            commands::reorder_saga_steps,
            commands::get_completions,
            commands::add_quest,
            commands::complete_quest,
            commands::set_quest_last_done,
            commands::update_quest,
            commands::delete_quest,
            commands::delete_completion,
            commands::reorder_list,
            commands::dismiss_quest_today,
            commands::get_character,
            commands::get_xp_stats,
            commands::update_character,
            commands::get_attributes,
            commands::get_skills,
            commands::get_quest_links,
            commands::set_quest_links,
            commands::reset_character,
            commands::reset_quests,
            commands::reset_completions,
            commands::get_quest_scores,
            commands::get_next_quest,
            commands::start_timer,
            commands::cancel_timer,
            commands::complete_timer,
            commands::get_timer_state,
            commands::get_settings,
            commands::set_cta_interval,
            commands::toggle_call_to_adventure,
            commands::set_debug_scoring,
            commands::skip_quest,
            commands::dismiss_overlay,
            commands::add_attribute,
            commands::add_skill,
            commands::rename_attribute,
            commands::reorder_attributes,
            commands::reorder_skills,
            commands::rename_skill,
            commands::update_skill_attribute,
            commands::delete_attribute,
            commands::delete_skill,
            commands::get_campaigns,
            commands::create_campaign,
            commands::rename_campaign,
            commands::delete_campaign,
            commands::check_campaign_progress,
            commands::get_tags,
            commands::add_tag,
            commands::delete_tag,
            commands::set_quest_tags,
            commands::get_accomplishments,
            commands::delete_accomplishment,
            commands::set_last_skipped,
            commands::get_last_skipped,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Next Quest");
}

fn format_countdown(remaining_ms: u64) -> String {
    let total_secs = (remaining_ms / 1000) as u64;
    let m = total_secs / 60;
    let s = total_secs % 60;
    format!("{:02}:{:02}", m, s)
}

fn update_tray_title(app: &tauri::AppHandle, title: &str) {
    if let Some(tray) = app.tray_by_id("nq_tray") {
        let _ = tray.set_title(Some(title));
    }
}

fn cta_poll_loop(app: tauri::AppHandle) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;

        // Read tray state (release lock before acquiring others)
        let (cta_on, next_fire) = {
            let tray_state = app.state::<AppTrayState>();
            let tray = tray_state.0.lock().unwrap();
            (tray.call_to_adventure, tray.cta_next_fire)
        };

        if !cta_on {
            update_tray_title(&app, "");
            continue;
        }

        if now < next_fire {
            update_tray_title(&app, &format_countdown(next_fire - now));
            continue;
        }

        // Check timer (separate lock)
        let timer_running = {
            let timer_state = app.state::<AppTimerState>();
            let timer = timer_state.0.lock().unwrap();
            timer.quest_id.is_some()
        };

        if timer_running {
            update_tray_title(&app, "");
            continue;
        }

        // Check window focus
        let window_focused = app
            .get_webview_window("main")
            .map(|w| w.is_focused().unwrap_or(false))
            .unwrap_or(false);

        if window_focused {
            continue;
        }

        // Check there's a quest to suggest
        let has_quest = {
            let db_state = app.state::<DbState>();
            let conn = db_state.0.lock().unwrap();
            db::get_next_quest(&conn, &std::collections::HashMap::new(), None, &db::Lane::CastleDuties)
                .map(|q| q.is_some())
                .unwrap_or(false)
        };

        if has_quest {
            update_tray_title(&app, "");
            show_overlay(&app);

            let tray_state = app.state::<AppTrayState>();
            let mut tray = tray_state.0.lock().unwrap();
            tray.reset_fire_time();
        }
    }
}

fn show_overlay(app: &tauri::AppHandle) {
    use tauri::WebviewWindowBuilder;

    if let Some(overlay) = app.get_webview_window("overlay") {
        // Already exists (hidden) — refresh, show, and force to front
        let _ = overlay.emit("overlay-refresh", ());
        let _ = overlay.show();
        let _ = overlay.center();
        let _ = overlay.set_always_on_top(true);
        let _ = overlay.set_focus();
        return;
    }

    // First time — create the window (hidden initially, then show)
    match WebviewWindowBuilder::new(
        app,
        "overlay",
        tauri::WebviewUrl::App("overlay.html".into()),
    )
    .title("")
    .inner_size(460.0, 400.0)
    .decorations(false)
    .always_on_top(true)
    .resizable(false)
    .minimizable(false)
    .accept_first_mouse(true)
    .center()
    .focused(true)
    .build()
    {
        Ok(_) => {}
        Err(e) => eprintln!("[CTA] Failed to create overlay: {}", e),
    }
}
