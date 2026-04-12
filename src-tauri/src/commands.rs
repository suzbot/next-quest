use nq_core::rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

use nq_core::db;

pub struct DbState(pub Mutex<Connection>);

#[derive(Default)]
pub struct TimerStateInner {
    pub quest_id: Option<String>,
    pub quest_title: Option<String>,
    pub started_at: Option<u64>,
    pub monster_image: Option<String>,
    pub encounter_line: Option<String>,
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

#[derive(Default)]
pub struct SkipStateInner {
    pub skip_counts: std::collections::HashMap<String, i32>,
    pub reset_date: String,
    pub last_skipped_id: Option<String>,
}

pub struct AppSkipState(pub Mutex<SkipStateInner>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerInfo {
    pub active: bool,
    pub quest_id: Option<String>,
    pub quest_title: Option<String>,
    pub started_at: Option<u64>,
    pub monster_image: Option<String>,
    pub encounter_line: Option<String>,
}

#[tauri::command]
pub fn get_quests(state: State<DbState>) -> Result<Vec<db::Quest>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_quests(&conn)
}

#[tauri::command]
pub fn get_quest_list(state: State<DbState>) -> Result<Vec<db::QuestListItem>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_quest_list(&conn)
}

// --- Saga commands ---

#[tauri::command]
pub fn get_sagas(state: State<DbState>) -> Result<Vec<db::Saga>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_sagas(&conn)
}

#[tauri::command]
pub fn add_saga(
    state: State<DbState>,
    name: String,
    cycle_days: Option<i32>,
) -> Result<db::Saga, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_saga(&conn, name, cycle_days)
}

#[tauri::command]
pub fn update_saga(
    state: State<DbState>,
    saga_id: String,
    name: Option<String>,
    saga_type: Option<String>,
    cycle_days: Option<i32>,
) -> Result<db::Saga, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::update_saga(&conn, saga_id, name, saga_type, cycle_days)
}

#[tauri::command]
pub fn delete_saga(
    state: State<DbState>,
    saga_id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_saga(&conn, saga_id)
}

#[tauri::command]
pub fn get_sagas_with_progress(
    state: State<DbState>,
) -> Result<Vec<db::SagaWithProgress>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_sagas_with_progress(&conn)
}

#[tauri::command]
pub fn check_saga_completion(
    state: State<DbState>,
    saga_id: String,
) -> Result<db::SagaCompletionResult, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::check_saga_completion(&conn, &saga_id)
}

#[tauri::command]
pub fn check_saga_completion_for_quest(
    state: State<DbState>,
    quest_id: String,
) -> Result<db::SagaCompletionResult, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::check_saga_completion_for_quest(&conn, &quest_id)
}

#[tauri::command]
pub fn get_saga_steps(
    state: State<DbState>,
    saga_id: String,
) -> Result<Vec<db::Quest>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_saga_steps(&conn, &saga_id)
}

#[tauri::command]
pub fn add_saga_step(
    state: State<DbState>,
    step: db::NewSagaStep,
) -> Result<db::Quest, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_saga_step(&conn, step)
}

#[tauri::command]
pub fn reorder_saga_steps(
    state: State<DbState>,
    saga_id: String,
    step_ids: Vec<String>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::reorder_saga_steps(&conn, &saga_id, step_ids)
}

// --- Campaign commands ---

#[tauri::command]
pub fn get_campaigns(state: State<DbState>) -> Result<Vec<db::CampaignWithCriteria>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_campaigns(&conn)
}

#[tauri::command]
pub fn create_campaign(
    state: State<DbState>,
    name: String,
    criteria: Vec<db::NewCriterion>,
) -> Result<db::CampaignWithCriteria, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::create_campaign(&conn, name, criteria)
}

#[tauri::command]
pub fn rename_campaign(
    state: State<DbState>,
    id: String,
    name: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::rename_campaign(&conn, id, name)
}

#[tauri::command]
pub fn delete_campaign(
    state: State<DbState>,
    id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_campaign(&conn, id)
}

#[tauri::command]
pub fn check_campaign_progress(
    state: State<DbState>,
    target_type: String,
    target_id: String,
) -> Result<Vec<db::CampaignCompletionResult>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::check_campaign_progress(&conn, &target_type, &target_id)
}

#[tauri::command]
pub fn get_accomplishments(state: State<DbState>) -> Result<Vec<db::Accomplishment>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_accomplishments(&conn)
}

#[tauri::command]
pub fn delete_accomplishment(
    state: State<DbState>,
    id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_accomplishment(&conn, id)
}

// --- Tags ---

#[tauri::command]
pub fn get_tags(state: State<DbState>) -> Result<Vec<db::Tag>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_tags(&conn)
}

#[tauri::command]
pub fn add_tag(state: State<DbState>, name: String) -> Result<db::Tag, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_tag(&conn, name)
}

#[tauri::command]
pub fn delete_tag(state: State<DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_tag(&conn, id)
}

#[tauri::command]
pub fn set_quest_tags(
    state: State<DbState>,
    quest_id: String,
    tag_ids: Vec<String>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::set_quest_tags(&conn, quest_id, tag_ids)
}

// --- Completions ---

#[tauri::command]
pub fn get_completions(state: State<DbState>) -> Result<Vec<db::Completion>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_completions(&conn)
}

#[tauri::command]
pub fn add_quest(
    state: State<DbState>,
    quest: db::NewQuest,
) -> Result<db::Quest, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_quest(&conn, quest)
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
pub fn set_quest_last_done(
    state: State<DbState>,
    quest_id: String,
    last_done: Option<String>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::set_quest_last_done(&conn, quest_id, last_done)
}

#[tauri::command]
pub fn update_quest(
    state: State<DbState>,
    quest_id: String,
    update: db::QuestUpdate,
) -> Result<db::Quest, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::update_quest(&conn, quest_id, update)
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
pub fn reorder_list(
    state: State<DbState>,
    items: Vec<db::ListReorderItem>,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::reorder_list(&conn, items)
}

#[tauri::command]
pub fn dismiss_quest_today(
    state: State<DbState>,
    quest_id: String,
) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::dismiss_quest_today(&conn, &quest_id)
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
pub fn get_xp_stats(state: State<DbState>) -> Result<db::XpStats, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::get_xp_stats(&conn)
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
pub fn get_quest_scores(
    state: State<DbState>,
    skip_state: State<AppSkipState>,
    lane: db::Lane,
) -> Result<Vec<db::ScoredQuest>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut skips = skip_state.0.lock().map_err(|e| e.to_string())?;
    let today = db::local_today_str();
    if skips.reset_date != today {
        skips.skip_counts.clear();
        skips.reset_date = today;
    }
    db::get_quest_scores(&conn, &skips.skip_counts, &lane)
}

#[tauri::command]
pub fn get_next_quest(
    state: State<DbState>,
    skip_state: State<AppSkipState>,
    exclude_quest_id: Option<String>,
    lane: db::Lane,
) -> Result<Option<db::ScoredQuest>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut skips = skip_state.0.lock().map_err(|e| e.to_string())?;
    // Daily reset
    let today = db::local_today_str();
    if skips.reset_date != today {
        skips.skip_counts.clear();
        skips.reset_date = today;
    }
    db::get_next_quest(&conn, &skips.skip_counts, exclude_quest_id.as_deref(), &lane)
}

#[tauri::command]
pub fn skip_quest(
    skip_state: State<AppSkipState>,
    quest_id: String,
) -> Result<(), String> {
    let mut skips = skip_state.0.lock().map_err(|e| e.to_string())?;
    let today = db::local_today_str();
    if skips.reset_date != today {
        skips.skip_counts.clear();
        skips.reset_date = today;
    }
    *skips.skip_counts.entry(quest_id).or_insert(0) += 1;
    Ok(())
}

#[tauri::command]
pub fn set_last_skipped(
    skip_state: State<AppSkipState>,
    quest_id: Option<String>,
) -> Result<(), String> {
    let mut skips = skip_state.0.lock().map_err(|e| e.to_string())?;
    skips.last_skipped_id = quest_id;
    Ok(())
}

#[tauri::command]
pub fn get_last_skipped(
    skip_state: State<AppSkipState>,
) -> Result<Option<String>, String> {
    let skips = skip_state.0.lock().map_err(|e| e.to_string())?;
    Ok(skips.last_skipped_id.clone())
}

#[tauri::command]
pub fn reset_skips(skip_state: State<AppSkipState>) -> Result<(), String> {
    let mut skips = skip_state.0.lock().map_err(|e| e.to_string())?;
    skips.skip_counts.clear();
    skips.last_skipped_id = None;
    Ok(())
}

// --- Timer ---

#[tauri::command]
pub fn start_timer(
    db_state: State<DbState>,
    timer_state: State<AppTimerState>,
    quest_id: String,
    monster_image: Option<String>,
    encounter_line: Option<String>,
) -> Result<TimerInfo, String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;

    // Look up quest title directly (not via get_quests, which excludes saga steps)
    let title: String = conn.query_row(
        "SELECT title FROM quest WHERE id = ?1",
        nq_core::rusqlite::params![quest_id],
        |row| row.get(0),
    ).map_err(|_| format!("Quest not found: {}", quest_id))?;

    let now_millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;

    let mut timer = timer_state.0.lock().map_err(|e| e.to_string())?;
    timer.quest_id = Some(quest_id.clone());
    timer.quest_title = Some(title.clone());
    timer.started_at = Some(now_millis);
    timer.monster_image = monster_image.clone();
    timer.encounter_line = encounter_line.clone();

    Ok(TimerInfo {
        active: true,
        quest_id: Some(quest_id),
        quest_title: Some(title),
        started_at: Some(now_millis),
        monster_image,
        encounter_line,
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
        monster_image: timer.monster_image.clone(),
        encounter_line: timer.encounter_line.clone(),
    })
}

// --- Settings ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInfo {
    pub call_to_adventure: bool,
    pub cta_interval_minutes: u64,
    pub debug_scoring: bool,
}

#[tauri::command]
pub fn get_settings(
    db_state: State<DbState>,
    tray_state: State<AppTrayState>,
) -> Result<SettingsInfo, String> {
    let tray = tray_state.0.lock().map_err(|e| e.to_string())?;
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    let (_, _, debug) = db::get_settings_db(&conn).unwrap_or((false, 20, false));
    Ok(SettingsInfo {
        call_to_adventure: tray.call_to_adventure,
        cta_interval_minutes: tray.cta_interval_secs / 60,
        debug_scoring: debug,
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
pub fn set_debug_scoring(
    db_state: State<DbState>,
    enabled: bool,
) -> Result<(), String> {
    let conn = db_state.0.lock().map_err(|e| e.to_string())?;
    db::set_debug_scoring(&conn, enabled)
}

#[tauri::command]
pub fn dismiss_overlay(
    app: tauri::AppHandle,
    tray_state: State<AppTrayState>,
    action: String,
) -> Result<(), String> {
    use tauri::Manager;

    // Hide overlay and disable always-on-top to prevent stale appearances
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.set_always_on_top(false);
        let _ = overlay.hide();
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
            // Deactivate the app so focus returns to whatever the user was working on,
            // rather than macOS activating the main window when the overlay hides.
            let _ = app.hide();
            let mut tray = tray_state.0.lock().map_err(|e| e.to_string())?;
            tray.reset_fire_time();
        }
        _ => {}
    }

    Ok(())
}

#[tauri::command]
pub fn add_attribute(state: State<DbState>, name: String) -> Result<db::Attribute, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_attribute(&conn, name)
}

#[tauri::command]
pub fn add_skill(state: State<DbState>, name: String, attribute_id: Option<String>) -> Result<db::Skill, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::add_skill(&conn, name, attribute_id)
}

#[tauri::command]
pub fn rename_attribute(state: State<DbState>, id: String, name: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::rename_attribute(&conn, id, name)
}

#[tauri::command]
pub fn reorder_attributes(state: State<DbState>, attr_ids: Vec<String>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::reorder_attributes(&conn, attr_ids)
}

#[tauri::command]
pub fn reorder_skills(state: State<DbState>, skill_ids: Vec<String>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::reorder_skills(&conn, skill_ids)
}

#[tauri::command]
pub fn rename_skill(state: State<DbState>, id: String, name: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::rename_skill(&conn, id, name)
}

#[tauri::command]
pub fn update_skill_attribute(state: State<DbState>, skill_id: String, attribute_id: Option<String>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::update_skill_attribute(&conn, skill_id, attribute_id)
}

#[tauri::command]
pub fn delete_attribute(state: State<DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_attribute(&conn, id)
}

#[tauri::command]
pub fn delete_skill(state: State<DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    db::delete_skill(&conn, id)
}
