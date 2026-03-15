use rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

use crate::db;

pub struct DbState(pub Mutex<Connection>);

#[derive(Default)]
pub struct TimerStateInner {
    pub quest_id: Option<String>,
    pub quest_title: Option<String>,
    pub started_at: Option<u64>,
}

pub struct AppTimerState(pub Mutex<TimerStateInner>);

#[derive(Default)]
pub struct TrayStateInner {
    pub call_to_adventure: bool,
    pub cta_interval_secs: u64,
    pub cta_next_fire: u64, // Unix millis — when to next show the overlay
}

impl TrayStateInner {
    pub fn new() -> Self {
        TrayStateInner {
            call_to_adventure: false,
            cta_interval_secs: 1200, // 20 minutes
            cta_next_fire: 0,
        }
    }

    pub fn reset_fire_time(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64;
        self.cta_next_fire = now + (self.cta_interval_secs * 1000);
    }
}

pub struct AppTrayState(pub Mutex<TrayStateInner>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerInfo {
    pub active: bool,
    pub quest_id: Option<String>,
    pub quest_title: Option<String>,
    pub started_at: Option<u64>,
}

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

// --- Quest Selection ---

#[tauri::command]
pub fn get_next_quest(
    state: State<DbState>,
    skip_count: i32,
) -> Result<Option<db::Quest>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_next_quest(&conn, skip_count)
}

// --- Timer ---

#[tauri::command]
pub fn start_timer(
    db_state: State<DbState>,
    timer_state: State<AppTimerState>,
    quest_id: String,
) -> Result<TimerInfo, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    // Look up quest title
    let quests = db::get_quests(&conn)?;
    let quest = quests.iter().find(|q| q.id == quest_id)
        .ok_or_else(|| format!("Quest not found: {}", quest_id))?;
    let title = quest.title.clone();

    let now_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;

    let mut timer = timer_state.0.lock().map_err(|e| e.to_string())?;
    timer.quest_id = Some(quest_id.clone());
    timer.quest_title = Some(title.clone());
    timer.started_at = Some(now_millis);

    Ok(TimerInfo {
        active: true,
        quest_id: Some(quest_id),
        quest_title: Some(title),
        started_at: Some(now_millis),
    })
}

#[tauri::command]
pub fn cancel_timer(
    timer_state: State<AppTimerState>,
    tray_state: State<AppTrayState>,
) -> Result<(), String> {
    let mut timer = timer_state.0.lock().map_err(|e| e.to_string())?;
    timer.quest_id = None;
    timer.quest_title = None;
    timer.started_at = None;
    // Reset CTA fire time so polling restarts
    let mut tray = tray_state.0.lock().map_err(|e| e.to_string())?;
    tray.reset_fire_time();
    Ok(())
}

#[tauri::command]
pub fn complete_timer(
    db_state: State<DbState>,
    timer_state: State<AppTimerState>,
    tray_state: State<AppTrayState>,
) -> Result<db::Completion, String> {
    let quest_id = {
        let timer = timer_state.0.lock().map_err(|e| e.to_string())?;
        timer.quest_id.clone().ok_or("No timer is active")?
    };

    // Complete the quest
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let completion = db::complete_quest(&conn, quest_id)?;

    // Clear timer
    let mut timer = timer_state.0.lock().map_err(|e| e.to_string())?;
    timer.quest_id = None;
    timer.quest_title = None;
    timer.started_at = None;

    // Reset CTA fire time so polling restarts
    let mut tray = tray_state.0.lock().map_err(|e| e.to_string())?;
    tray.reset_fire_time();

    Ok(completion)
}

#[tauri::command]
pub fn get_timer_state(
    timer_state: State<AppTimerState>,
) -> Result<TimerInfo, String> {
    let timer = timer_state.0.lock().map_err(|e| e.to_string())?;
    Ok(TimerInfo {
        active: timer.quest_id.is_some(),
        quest_id: timer.quest_id.clone(),
        quest_title: timer.quest_title.clone(),
        started_at: timer.started_at,
    })
}

// --- Settings ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInfo {
    pub call_to_adventure: bool,
    pub cta_interval_minutes: u64,
}

#[tauri::command]
pub fn get_settings(
    tray_state: State<AppTrayState>,
) -> Result<SettingsInfo, String> {
    let tray = tray_state.0.lock().map_err(|e| e.to_string())?;
    Ok(SettingsInfo {
        call_to_adventure: tray.call_to_adventure,
        cta_interval_minutes: tray.cta_interval_secs / 60,
    })
}

#[tauri::command]
pub fn toggle_call_to_adventure(
    app: tauri::AppHandle,
    db_state: State<DbState>,
    tray_state: State<AppTrayState>,
) -> Result<bool, String> {
    let (new_val, interval_mins) = {
        let mut tray = tray_state.0.lock().map_err(|e| e.to_string())?;
        tray.call_to_adventure = !tray.call_to_adventure;
        if tray.call_to_adventure {
            tray.reset_fire_time();
        }
        (tray.call_to_adventure, tray.cta_interval_secs / 60)
    };
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    db::set_settings_db(&conn, new_val, interval_mins)?;
    crate::tray::rebuild_tray_menu(&app);
    Ok(new_val)
}

#[tauri::command]
pub fn set_cta_interval(
    db_state: State<DbState>,
    tray_state: State<AppTrayState>,
    minutes: u64,
) -> Result<(), String> {
    let minutes = if minutes < 1 { 1 } else { minutes };
    let (enabled, _) = {
        let tray = tray_state.0.lock().map_err(|e| e.to_string())?;
        (tray.call_to_adventure, ())
    };
    let mut tray = tray_state.0.lock().map_err(|e| e.to_string())?;
    tray.cta_interval_secs = minutes * 60;
    drop(tray);
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    db::set_settings_db(&conn, enabled, minutes)?;
    Ok(())
}

#[tauri::command]
pub fn dismiss_overlay(
    app: tauri::AppHandle,
    tray_state: State<AppTrayState>,
    action: String,
) -> Result<(), String> {
    use tauri::Manager;

    // Move overlay off-screen (avoids macOS focusing the main window)
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.set_position(tauri::LogicalPosition::new(-9999.0, -9999.0));
    }

    match action.as_str() {
        "quest_now" => {
            // Timer is started by the overlay frontend via start_timer.
            // Just show the main window — it detects the running timer on focus.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
            let mut tray = tray_state.0.lock().map_err(|e| e.to_string())?;
            tray.reset_fire_time();
        }
        "later" => {
            let mut tray = tray_state.0.lock().map_err(|e| e.to_string())?;
            tray.reset_fire_time();
        }
        _ => {}
    }

    Ok(())
}

