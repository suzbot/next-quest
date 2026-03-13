#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;

use commands::{AppTimerState, DbState, TimerStateInner};
use std::sync::Mutex;

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Next Quest");
}
