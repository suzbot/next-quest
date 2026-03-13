use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

use crate::db;

pub struct DbState(pub Mutex<Connection>);

#[tauri::command]
pub fn get_quests(state: State<DbState>) -> Result<Vec<db::Quest>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_quests(&conn)
}

#[tauri::command]
pub fn add_quest(
    state: State<DbState>,
    title: String,
    cycle_days: Option<i32>,
) -> Result<db::Quest, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_quest(&conn, title, cycle_days)
}
