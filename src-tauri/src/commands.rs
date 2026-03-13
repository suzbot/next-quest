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
    difficulty: db::Difficulty,
) -> Result<db::Quest, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_quest(&conn, title, quest_type, cycle_days, difficulty)
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
    difficulty: Option<db::Difficulty>,
) -> Result<db::Quest, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::update_quest(&conn, quest_id, title, quest_type, cycle_days, difficulty)
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

#[tauri::command]
pub fn get_character(state: State<DbState>) -> Result<db::Character, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_character(&conn)
}

#[tauri::command]
pub fn update_character(
    state: State<DbState>,
    name: String,
) -> Result<db::Character, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::update_character(&conn, name)
}

#[tauri::command]
pub fn get_attributes(state: State<DbState>) -> Result<Vec<db::Attribute>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_attributes(&conn)
}

#[tauri::command]
pub fn get_skills(state: State<DbState>) -> Result<Vec<db::Skill>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_skills(&conn)
}

#[tauri::command]
pub fn get_quest_links(
    state: State<DbState>,
    quest_id: String,
) -> Result<db::QuestLinks, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_quest_links(&conn, quest_id)
}

#[tauri::command]
pub fn set_quest_links(
    state: State<DbState>,
    quest_id: String,
    skill_ids: Vec<String>,
    attribute_ids: Vec<String>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::set_quest_links(&conn, quest_id, skill_ids, attribute_ids)
}

#[tauri::command]
pub fn reset_character(state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::reset_character(&conn)
}

#[tauri::command]
pub fn reset_quests(state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::reset_quests(&conn)
}

#[tauri::command]
pub fn reset_completions(state: State<DbState>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::reset_completions(&conn)
}
