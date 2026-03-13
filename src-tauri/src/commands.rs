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
pub fn get_completions(state: State<DbState>) -> Result<Vec<db::Completion>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_completions(&conn)
}

#[tauri::command]
pub fn add_quest(
    state: State<DbState>,
    title: String,
    quest_type: db::QuestType,
    cycle_days: Option<i32>,
) -> Result<db::Quest, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_quest(&conn, title, quest_type, cycle_days)
}

#[tauri::command]
pub fn complete_quest(
    state: State<DbState>,
    quest_id: String,
) -> Result<db::Completion, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::complete_quest(&conn, quest_id)
}

#[tauri::command]
pub fn update_quest(
    state: State<DbState>,
    quest_id: String,
    title: Option<String>,
    quest_type: Option<db::QuestType>,
    cycle_days: Option<i32>,
) -> Result<db::Quest, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::update_quest(&conn, quest_id, title, quest_type, cycle_days)
}

#[tauri::command]
pub fn delete_quest(
    state: State<DbState>,
    quest_id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_quest(&conn, quest_id)
}

#[tauri::command]
pub fn reorder_quests(
    state: State<DbState>,
    orders: Vec<db::QuestOrder>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::reorder_quests(&conn, orders)
}

#[tauri::command]
pub fn delete_completion(
    state: State<DbState>,
    completion_id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_completion(&conn, completion_id)
}
