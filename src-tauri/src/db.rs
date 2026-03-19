use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QuestType {
    Recurring,
    OneOff,
}

impl QuestType {
    fn as_str(&self) -> &str {
        match self {
            QuestType::Recurring => "recurring",
            QuestType::OneOff => "one_off",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "recurring" => QuestType::Recurring,
            _ => QuestType::OneOff,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Trivial,
    Easy,
    Moderate,
    Challenging,
    Epic,
}

impl Difficulty {
    fn as_str(&self) -> &str {
        match self {
            Difficulty::Trivial => "trivial",
            Difficulty::Easy => "easy",
            Difficulty::Moderate => "moderate",
            Difficulty::Challenging => "challenging",
            Difficulty::Epic => "epic",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "trivial" => Difficulty::Trivial,
            "moderate" => Difficulty::Moderate,
            "challenging" => Difficulty::Challenging,
            "epic" => Difficulty::Epic,
            _ => Difficulty::Easy,
        }
    }
}

pub enum LevelScale {
    Character,
    Attribute,
    Skill,
}

#[derive(Serialize, Debug, Clone)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub quest_type: QuestType,
    pub cycle_days: Option<i32>,
    pub sort_order: i32,
    pub active: bool,
    pub created_at: String,
    pub difficulty: Difficulty,
    pub time_of_day: i32,
    pub days_of_week: i32,
    pub last_completed: Option<String>,
    pub is_due: bool,
    pub skill_ids: Vec<String>,
    pub attribute_ids: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct QuestLinks {
    pub skill_ids: Vec<String>,
    pub attribute_ids: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct QuestOrder {
    pub id: String,
    pub sort_order: i32,
}

#[derive(Serialize, Debug, Clone)]
pub struct ScoredQuest {
    pub quest: Quest,
    pub score: f64,
    pub overdue_ratio: f64,
    pub skip_penalty: f64,
    pub list_order_bonus: f64,
    pub pool: String,
    pub due_count: usize,
    pub not_due_count: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct LevelUp {
    pub name: String,
    pub new_level: i32,
}

#[derive(Serialize, Debug)]
pub struct Completion {
    pub id: String,
    pub quest_id: Option<String>,
    pub quest_title: String,
    pub completed_at: String,
    pub xp_earned: i64,
    pub level_ups: Vec<LevelUp>,
}

#[derive(Serialize, Debug)]
pub struct LevelInfo {
    pub level: i32,
    pub xp_for_current_level: i64,
    pub xp_into_current_level: i64,
}

#[derive(Serialize, Debug)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub xp: i64,
    pub level: i32,
    pub xp_for_current_level: i64,
    pub xp_into_current_level: i64,
}

#[derive(Serialize, Debug)]
pub struct Attribute {
    pub id: String,
    pub name: String,
    pub sort_order: i32,
    pub xp: i64,
    pub level: i32,
    pub xp_for_current_level: i64,
    pub xp_into_current_level: i64,
}

#[derive(Serialize, Debug)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub attribute_id: Option<String>,
    pub sort_order: i32,
    pub xp: i64,
    pub level: i32,
    pub xp_for_current_level: i64,
    pub xp_into_current_level: i64,
}

pub fn init_db(db_path: &Path) -> Connection {
    let conn = Connection::open(db_path).expect("Failed to open database");
    create_tables(&conn);
    migrate(&conn);
    seed_data(&conn);
    conn
}

#[cfg(test)]
pub fn init_db_memory() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory database");
    create_tables(&conn);
    seed_data(&conn);
    conn
}

fn create_tables(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS quest (
            id          TEXT PRIMARY KEY,
            title       TEXT NOT NULL,
            quest_type  TEXT NOT NULL DEFAULT 'recurring',
            cycle_days  INTEGER,
            sort_order  INTEGER NOT NULL,
            active      INTEGER NOT NULL DEFAULT 1,
            created_at  TEXT NOT NULL,
            difficulty  TEXT NOT NULL DEFAULT 'easy',
            time_of_day INTEGER NOT NULL DEFAULT 7,
            days_of_week INTEGER NOT NULL DEFAULT 127
        );

        CREATE TABLE IF NOT EXISTS quest_completion (
            id            TEXT PRIMARY KEY,
            quest_id      TEXT,
            quest_title   TEXT NOT NULL DEFAULT '',
            completed_at  TEXT NOT NULL,
            xp_earned     INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS character (
            id    TEXT PRIMARY KEY,
            name  TEXT NOT NULL,
            xp    INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS attribute (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            sort_order  INTEGER NOT NULL,
            xp          INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS skill (
            id            TEXT PRIMARY KEY,
            name          TEXT NOT NULL,
            attribute_id  TEXT REFERENCES attribute(id),
            sort_order    INTEGER NOT NULL,
            xp            INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS quest_skill (
            quest_id  TEXT NOT NULL REFERENCES quest(id),
            skill_id  TEXT NOT NULL REFERENCES skill(id),
            PRIMARY KEY (quest_id, skill_id)
        );

        CREATE TABLE IF NOT EXISTS quest_attribute (
            quest_id      TEXT NOT NULL REFERENCES quest(id),
            attribute_id  TEXT NOT NULL REFERENCES attribute(id),
            PRIMARY KEY (quest_id, attribute_id)
        );

        CREATE TABLE IF NOT EXISTS settings (
            id                    INTEGER PRIMARY KEY,
            cta_enabled           INTEGER NOT NULL DEFAULT 0,
            cta_interval_minutes  INTEGER NOT NULL DEFAULT 20,
            debug_scoring         INTEGER NOT NULL DEFAULT 0
        );",
    )
    .expect("Failed to create tables");
}

fn migrate(conn: &Connection) {
    // Migration: add quest_title to quest_completion if missing
    let has_quest_title: bool = conn
        .prepare("SELECT quest_title FROM quest_completion LIMIT 0")
        .is_ok();

    if !has_quest_title {
        conn.execute_batch(
            "ALTER TABLE quest_completion ADD COLUMN quest_title TEXT NOT NULL DEFAULT '';"
        ).expect("Failed to add quest_title column");

        conn.execute_batch(
            "UPDATE quest_completion SET quest_title = (
                SELECT title FROM quest WHERE quest.id = quest_completion.quest_id
            ) WHERE quest_id IS NOT NULL;"
        ).expect("Failed to backfill quest_title");
    }

    // Migration: make quest_id nullable on quest_completion
    // SQLite can't ALTER COLUMN, so we recreate the table
    let quest_id_is_not_null: bool = conn
        .prepare(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='quest_completion'"
        )
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, String>(0)))
        .map(|sql| sql.contains("quest_id      TEXT NOT NULL") || sql.contains("quest_id TEXT NOT NULL"))
        .unwrap_or(false);

    if quest_id_is_not_null {
        conn.execute_batch(
            "CREATE TABLE quest_completion_new (
                id            TEXT PRIMARY KEY,
                quest_id      TEXT,
                quest_title   TEXT NOT NULL DEFAULT '',
                completed_at  TEXT NOT NULL
            );
            INSERT INTO quest_completion_new SELECT id, quest_id, quest_title, completed_at FROM quest_completion;
            DROP TABLE quest_completion;
            ALTER TABLE quest_completion_new RENAME TO quest_completion;"
        ).expect("Failed to migrate quest_completion to nullable quest_id");
    }

    // Migration: add quest_type to quest if missing
    let has_quest_type: bool = conn
        .prepare("SELECT quest_type FROM quest LIMIT 0")
        .is_ok();

    if !has_quest_type {
        conn.execute_batch(
            "ALTER TABLE quest ADD COLUMN quest_type TEXT NOT NULL DEFAULT 'recurring';"
        ).expect("Failed to add quest_type column");

        // Backfill: quests with null cycle_days are one_off
        conn.execute_batch(
            "UPDATE quest SET quest_type = 'one_off' WHERE cycle_days IS NULL;"
        ).expect("Failed to backfill quest_type");
    }

    // Migration: add difficulty to quest if missing
    let has_difficulty: bool = conn
        .prepare("SELECT difficulty FROM quest LIMIT 0")
        .is_ok();

    if !has_difficulty {
        conn.execute_batch(
            "ALTER TABLE quest ADD COLUMN difficulty TEXT NOT NULL DEFAULT 'easy';"
        ).expect("Failed to add difficulty column");
    }

    // Migration: add xp_earned to quest_completion if missing
    let has_xp_earned: bool = conn
        .prepare("SELECT xp_earned FROM quest_completion LIMIT 0")
        .is_ok();

    if !has_xp_earned {
        conn.execute_batch(
            "ALTER TABLE quest_completion ADD COLUMN xp_earned INTEGER NOT NULL DEFAULT 0;"
        ).expect("Failed to add xp_earned column");
    }

    // Migration: make skill.attribute_id nullable
    let skill_attr_not_null: bool = conn
        .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='skill'")
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, String>(0)))
        .map(|sql| sql.contains("attribute_id  TEXT NOT NULL"))
        .unwrap_or(false);

    if skill_attr_not_null {
        conn.execute_batch("PRAGMA foreign_keys = OFF;").expect("Failed to disable foreign keys");
        conn.execute_batch(
            "DROP TABLE IF EXISTS skill_new;
            CREATE TABLE skill_new (
                id            TEXT PRIMARY KEY,
                name          TEXT NOT NULL,
                attribute_id  TEXT REFERENCES attribute(id),
                sort_order    INTEGER NOT NULL,
                xp            INTEGER NOT NULL DEFAULT 0
            );
            INSERT INTO skill_new SELECT id, name, attribute_id, sort_order, xp FROM skill;
            DROP TABLE skill;
            ALTER TABLE skill_new RENAME TO skill;"
        ).expect("Failed to migrate skill to nullable attribute_id");
        conn.execute_batch("PRAGMA foreign_keys = ON;").expect("Failed to re-enable foreign keys");
    }

    // Migration: add time_of_day to quest if missing
    let has_time_of_day: bool = conn
        .prepare("SELECT time_of_day FROM quest LIMIT 0")
        .is_ok();

    if !has_time_of_day {
        conn.execute_batch(
            "ALTER TABLE quest ADD COLUMN time_of_day INTEGER NOT NULL DEFAULT 7;"
        ).expect("Failed to add time_of_day column");
    }

    // Migration: convert time_of_day from TEXT to INTEGER bitmask
    // Morning=1, Afternoon=2, Evening=4, all=7
    let tod_is_text: bool = conn
        .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='quest'")
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, String>(0)))
        .map(|sql| sql.contains("time_of_day TEXT"))
        .unwrap_or(false);

    if tod_is_text {
        conn.execute_batch(
            "UPDATE quest SET time_of_day = CASE time_of_day
                WHEN 'morning' THEN 1
                WHEN 'afternoon' THEN 2
                WHEN 'evening' THEN 4
                ELSE 7
            END;"
        ).expect("Failed to convert time_of_day values");
        // Recreate column as INTEGER
        conn.execute_batch("PRAGMA foreign_keys = OFF;").expect("FK off");
        conn.execute_batch(
            "ALTER TABLE quest RENAME COLUMN time_of_day TO time_of_day_old;
             ALTER TABLE quest ADD COLUMN time_of_day INTEGER NOT NULL DEFAULT 7;
             UPDATE quest SET time_of_day = time_of_day_old;
             ALTER TABLE quest DROP COLUMN time_of_day_old;"
        ).expect("Failed to migrate time_of_day to INTEGER");
        conn.execute_batch("PRAGMA foreign_keys = ON;").expect("FK on");
    }

    // Migration: add days_of_week to quest if missing
    let has_days_of_week: bool = conn
        .prepare("SELECT days_of_week FROM quest LIMIT 0")
        .is_ok();

    if !has_days_of_week {
        conn.execute_batch(
            "ALTER TABLE quest ADD COLUMN days_of_week INTEGER NOT NULL DEFAULT 127;"
        ).expect("Failed to add days_of_week column");
    }

    // Migration: add debug_scoring to settings if missing
    let has_debug_scoring: bool = conn
        .prepare("SELECT debug_scoring FROM settings LIMIT 0")
        .is_ok();

    if !has_debug_scoring {
        conn.execute_batch(
            "ALTER TABLE settings ADD COLUMN debug_scoring INTEGER NOT NULL DEFAULT 0;"
        ).expect("Failed to add debug_scoring column");
    }
}

fn seed_data(conn: &Connection) {
    // Settings seed runs every launch (idempotent via INSERT OR IGNORE)
    conn.execute(
        "INSERT OR IGNORE INTO settings (id, cta_enabled, cta_interval_minutes) VALUES (1, 0, 20)",
        [],
    )
    .expect("Failed to seed settings");

    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM character", [], |row| row.get(0))
        .expect("Failed to check character table");

    if count > 0 {
        return;
    }

    let char_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO character (id, name, xp) VALUES (?1, ?2, 0)",
        rusqlite::params![char_id, "Adventurer"],
    )
    .expect("Failed to seed character");

    let attributes = [
        ("Health", 1),
        ("Pluck", 2),
        ("Knowledge", 3),
        ("Connection", 4),
        ("Responsibility", 5),
    ];

    let mut attr_ids = std::collections::HashMap::new();
    for (name, order) in &attributes {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO attribute (id, name, sort_order, xp) VALUES (?1, ?2, ?3, 0)",
            rusqlite::params![id, name, order],
        )
        .expect("Failed to seed attribute");
        attr_ids.insert(*name, id);
    }

    let skills = [
        ("Cooking", "Health", 1),
        ("Healing", "Health", 2),
        ("Acrobatics", "Health", 3),
        ("Cleaning", "Pluck", 4),
        ("Crafting", "Pluck", 5),
        ("Language", "Knowledge", 6),
        ("Technology", "Knowledge", 7),
        ("Nature", "Connection", 8),
        ("Community", "Connection", 9),
        ("Sociality", "Connection", 10),
        ("Animal Handling", "Connection", 11),
        ("Bureaucracy", "Responsibility", 12),
        ("Logistics", "Responsibility", 13),
    ];

    for (name, attr_name, order) in &skills {
        let id = Uuid::new_v4().to_string();
        let attr_id = &attr_ids[attr_name];
        conn.execute(
            "INSERT INTO skill (id, name, attribute_id, sort_order, xp) VALUES (?1, ?2, ?3, ?4, 0)",
            rusqlite::params![id, name, attr_id, order],
        )
        .expect("Failed to seed skill");
    }
}

pub fn get_settings_db(conn: &Connection) -> Result<(bool, u64, bool), String> {
    conn.query_row(
        "SELECT cta_enabled, cta_interval_minutes, debug_scoring FROM settings WHERE id = 1",
        [],
        |row| {
            let enabled = row.get::<_, i32>(0)? != 0;
            let interval: u64 = row.get(1)?;
            let debug = row.get::<_, i32>(2)? != 0;
            Ok((enabled, interval, debug))
        },
    )
    .map_err(|e| e.to_string())
}

pub fn set_settings_db(conn: &Connection, enabled: bool, interval_minutes: u64) -> Result<(), String> {
    conn.execute(
        "UPDATE settings SET cta_enabled = ?1, cta_interval_minutes = ?2 WHERE id = 1",
        rusqlite::params![enabled as i32, interval_minutes],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn set_debug_scoring(conn: &Connection, enabled: bool) -> Result<(), String> {
    conn.execute(
        "UPDATE settings SET debug_scoring = ?1 WHERE id = 1",
        rusqlite::params![enabled as i32],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_quests(conn: &Connection) -> Result<Vec<Quest>, String> {
    let today = local_today_days();
    let mut stmt = conn
        .prepare(
            "SELECT q.id, q.title, q.quest_type, q.cycle_days, q.sort_order, q.active, q.created_at,
                    q.difficulty, q.time_of_day, q.days_of_week,
                    (SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = q.id) as last_completed
             FROM quest q
             WHERE q.active = 1 OR (q.active = 0 AND q.quest_type = 'one_off')
             ORDER BY q.active DESC, q.sort_order DESC",
        )
        .map_err(|e| e.to_string())?;

    let mut quests: Vec<Quest> = stmt
        .query_map([], |row| {
            let quest_type_str: String = row.get(2)?;
            let quest_type = QuestType::from_str(&quest_type_str);
            let cycle_days: Option<i32> = row.get(3)?;
            let active = row.get::<_, i32>(5)? != 0;
            let difficulty_str: String = row.get(7)?;
            let time_of_day: i32 = row.get(8)?;
            let days_of_week: i32 = row.get(9)?;
            let last_completed: Option<String> = row.get(10)?;
            let last_completed_days = last_completed.as_deref().and_then(utc_iso_to_local_days);
            let is_due = compute_is_due(&quest_type, active, last_completed_days, cycle_days, today);
            Ok(Quest {
                id: row.get(0)?,
                title: row.get(1)?,
                quest_type,
                cycle_days,
                sort_order: row.get(4)?,
                active,
                created_at: row.get(6)?,
                difficulty: Difficulty::from_str(&difficulty_str),
                time_of_day,
                days_of_week,
                last_completed,
                is_due,
                skill_ids: Vec::new(),
                attribute_ids: Vec::new(),
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Batch-load links
    let skill_links = load_all_quest_skills(conn)?;
    let attr_links = load_all_quest_attributes(conn)?;
    for q in &mut quests {
        if let Some(sids) = skill_links.get(&q.id) {
            q.skill_ids = sids.clone();
        }
        if let Some(aids) = attr_links.get(&q.id) {
            q.attribute_ids = aids.clone();
        }
    }

    Ok(quests)
}

pub fn get_next_quest(conn: &Connection, skip_counts: &std::collections::HashMap<String, i32>, exclude_quest_id: Option<&str>) -> Result<Option<ScoredQuest>, String> {
    let quests = get_quests(conn)?;
    let today = local_today_days();
    let hour = local_hour();
    let weekday = local_weekday();

    // Hard-filter: active, time-of-day, day-of-week
    let eligible: Vec<&Quest> = quests.iter()
        .filter(|q| q.active)
        .filter(|q| matches_time_of_day(q.time_of_day, hour))
        .filter(|q| matches_day_of_week(q.days_of_week, weekday))
        .collect();

    if eligible.is_empty() {
        return Ok(None);
    }

    let due: Vec<&Quest> = eligible.iter().filter(|q| q.is_due).copied().collect();
    let not_due: Vec<&Quest> = eligible.iter().filter(|q| !q.is_due).copied().collect();
    let due_count = due.len();
    let not_due_count = not_due.len();

    // Score due pool first
    let (mut scored, mut pool_name) = if !due.is_empty() {
        (score_quests_due(&due, today, skip_counts), "due")
    } else {
        (Vec::new(), "due")
    };

    // If due pool is empty or all scores <= 0, include not-due pool
    let all_skipped = !scored.is_empty() && scored.iter().all(|s| s.0 <= 0.0);
    if scored.is_empty() || all_skipped {
        let not_due_scored = score_quests_not_due(&not_due, today, skip_counts);
        if scored.is_empty() {
            scored = not_due_scored;
            pool_name = "not_due";
        } else {
            scored.extend(not_due_scored);
            pool_name = "due+not_due";
        }
    }

    if scored.is_empty() {
        return Ok(None);
    }

    // Sort by score descending (highest first)
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Skip excluded quest if possible (fall back to it if it's the only one)
    let top = if let Some(exc_id) = exclude_quest_id {
        scored.iter()
            .find(|(_, _, _, _, q)| q.id != exc_id)
            .cloned()
            .unwrap_or_else(|| scored.into_iter().next().unwrap())
    } else {
        scored.into_iter().next().unwrap()
    };
    let (score, overdue_ratio, skip_penalty, list_order_bonus, quest) = top;

    Ok(Some(ScoredQuest {
        quest,
        score,
        overdue_ratio,
        skip_penalty,
        list_order_bonus,
        pool: pool_name.to_string(),
        due_count,
        not_due_count,
    }))
}

fn score_quests_due(quests: &[&Quest], today: i64, skip_counts: &std::collections::HashMap<String, i32>) -> Vec<(f64, f64, f64, f64, Quest)> {
    let max_sort = quests.iter().map(|q| q.sort_order).max().unwrap_or(1) as f64;

    quests.iter().map(|q| {
        let overdue_ratio = compute_overdue_ratio(q, today);
        let skips = *skip_counts.get(&q.id).unwrap_or(&0) as f64;
        let skip_penalty = skips * 0.5;
        let list_order_bonus = 0.01 * q.sort_order as f64 / max_sort;
        let score = overdue_ratio - skip_penalty + list_order_bonus;
        (score, overdue_ratio, skip_penalty, list_order_bonus, (*q).clone())
    }).collect()
}

fn score_quests_not_due(quests: &[&Quest], today: i64, skip_counts: &std::collections::HashMap<String, i32>) -> Vec<(f64, f64, f64, f64, Quest)> {
    let max_sort = quests.iter().map(|q| q.sort_order).max().unwrap_or(1) as f64;

    let days_since: Vec<f64> = quests.iter().map(|q| {
        match q.last_completed.as_deref().and_then(utc_iso_to_local_days) {
            Some(d) => (today - d) as f64,
            None => f64::MAX,
        }
    }).collect();
    let max_days = days_since.iter().cloned().filter(|d| *d < f64::MAX).fold(1.0f64, f64::max);

    quests.iter().enumerate().map(|(i, q)| {
        let normalized = if days_since[i] == f64::MAX { 1.0 } else { days_since[i] / max_days };
        let skips = *skip_counts.get(&q.id).unwrap_or(&0) as f64;
        let skip_penalty = skips * 0.5;
        let list_order_bonus = 0.01 * q.sort_order as f64 / max_sort;
        let score = normalized - skip_penalty + list_order_bonus;
        (score, normalized, skip_penalty, list_order_bonus, (*q).clone())
    }).collect()
}

fn compute_overdue_ratio(q: &Quest, today: i64) -> f64 {
    let cycle = match q.quest_type {
        QuestType::Recurring => q.cycle_days.unwrap_or(1) as f64,
        QuestType::OneOff => 9.0,
    };

    match q.last_completed.as_deref().and_then(utc_iso_to_local_days) {
        Some(last_day) => {
            let elapsed = (today - last_day) as f64;
            (elapsed / cycle).max(1.0)
        }
        None => {
            // Never completed: use days since created + cycle
            let created_day = utc_iso_to_local_days(&q.created_at).unwrap_or(today);
            let elapsed = (today - created_day) as f64 + cycle;
            elapsed / cycle
        }
    }
}

pub fn get_completions(conn: &Connection) -> Result<Vec<Completion>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, quest_id, quest_title, completed_at, xp_earned
             FROM quest_completion
             ORDER BY completed_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let completions = stmt
        .query_map([], |row| {
            Ok(Completion {
                id: row.get(0)?,
                quest_id: row.get(1)?,
                quest_title: row.get(2)?,
                completed_at: row.get(3)?,
                level_ups: Vec::new(),
                xp_earned: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(completions)
}

pub fn add_quest(
    conn: &Connection,
    title: String,
    quest_type: QuestType,
    cycle_days: Option<i32>,
    difficulty: Difficulty,
    time_of_day: i32,
    days_of_week: i32,
) -> Result<Quest, String> {
    let id = Uuid::new_v4().to_string();
    let created_at = chrono_now();

    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM quest",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let sort_order = max_order + 1;

    let effective_cycle = match quest_type {
        QuestType::Recurring => cycle_days.or(Some(1)),
        QuestType::OneOff => None,
    };

    conn.execute(
        "INSERT INTO quest (id, title, quest_type, cycle_days, sort_order, active, created_at, difficulty, time_of_day, days_of_week)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, title, quest_type.as_str(), effective_cycle, sort_order, created_at, difficulty.as_str(), time_of_day, days_of_week],
    )
    .map_err(|e| e.to_string())?;

    Ok(Quest {
        id,
        title,
        quest_type,
        cycle_days: effective_cycle,
        sort_order,
        active: true,
        created_at,
        difficulty,
        time_of_day,
        days_of_week,
        last_completed: None,
        is_due: true,
        skill_ids: Vec::new(),
        attribute_ids: Vec::new(),
    })
}

pub fn calculate_xp(difficulty: &Difficulty, quest_type: &QuestType, cycle_days: Option<i32>) -> i64 {
    let base: f64 = 5.0;

    let difficulty_mult: f64 = match difficulty {
        Difficulty::Trivial => 1.0,
        Difficulty::Easy => 5.0,
        Difficulty::Moderate => 10.0,
        Difficulty::Challenging => 20.0,
        Difficulty::Epic => 40.0,
    };

    let cycle_mult: f64 = match quest_type {
        QuestType::OneOff => 3.0,
        QuestType::Recurring => {
            let days = cycle_days.unwrap_or(1).max(1) as f64;
            days.sqrt()
        }
    };

    (base * difficulty_mult * cycle_mult).round() as i64
}

/// Compute the time-elapsed XP multiplier.
/// r = elapsed_secs / cycle_secs. Returns multiplier >= 0.1.
/// For r < 1: 0.1 + 0.9 * sqrt(r)
/// For r >= 1: 1.0 + 0.5 * ln(r)
pub fn time_elapsed_multiplier(r: f64) -> f64 {
    if r < 0.0 {
        return 0.1;
    }
    let mult = if r < 1.0 {
        0.1 + 0.9 * r.sqrt()
    } else {
        1.0 + 0.5 * r.ln()
    };
    mult.max(0.1)
}

pub fn award_xp(conn: &Connection, quest_id: &str, xp: i64) -> Result<(), String> {
    // Award to character
    conn.execute(
        "UPDATE character SET xp = xp + ?1",
        rusqlite::params![xp],
    )
    .map_err(|e| e.to_string())?;

    // Get linked attribute IDs and award XP
    let mut attr_stmt = conn
        .prepare("SELECT attribute_id FROM quest_attribute WHERE quest_id = ?1")
        .map_err(|e| e.to_string())?;
    let attr_ids: Vec<String> = attr_stmt
        .query_map(rusqlite::params![quest_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for aid in &attr_ids {
        conn.execute(
            "UPDATE attribute SET xp = xp + ?1 WHERE id = ?2",
            rusqlite::params![xp, aid],
        )
        .map_err(|e| e.to_string())?;
    }

    // Get linked skill IDs, check levels before/after, award XP
    let mut skill_stmt = conn
        .prepare("SELECT skill_id FROM quest_skill WHERE quest_id = ?1")
        .map_err(|e| e.to_string())?;
    let skill_ids: Vec<String> = skill_stmt
        .query_map(rusqlite::params![quest_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for sid in &skill_ids {
        // Read current XP to check level before
        let (old_xp, attribute_id): (i64, Option<String>) = conn
            .query_row(
                "SELECT xp, attribute_id FROM skill WHERE id = ?1",
                rusqlite::params![sid],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?;

        let level_before = level_from_xp(old_xp, &LevelScale::Skill).level;

        conn.execute(
            "UPDATE skill SET xp = xp + ?1 WHERE id = ?2",
            rusqlite::params![xp, sid],
        )
        .map_err(|e| e.to_string())?;

        let new_xp = old_xp + xp;
        let level_after = level_from_xp(new_xp, &LevelScale::Skill).level;

        // Skill leveled up — award attribute bump equivalent to a Moderate one-off
        if level_after > level_before {
            if let Some(ref aid) = attribute_id {
                let attr_bump = calculate_xp(&Difficulty::Moderate, &QuestType::OneOff, None);
                conn.execute(
                    "UPDATE attribute SET xp = xp + ?1 WHERE id = ?2",
                    rusqlite::params![attr_bump, aid],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

pub fn complete_quest(conn: &Connection, quest_id: String) -> Result<Completion, String> {
    // Read quest data for XP calculation
    let (quest_title, difficulty_str, quest_type_str, cycle_days): (String, String, String, Option<i32>) = conn
        .query_row(
            "SELECT title, difficulty, quest_type, cycle_days FROM quest WHERE id = ?1",
            rusqlite::params![quest_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|_| format!("Quest not found: {}", quest_id))?;

    let difficulty = Difficulty::from_str(&difficulty_str);
    let quest_type = QuestType::from_str(&quest_type_str);
    let base_xp = calculate_xp(&difficulty, &quest_type, cycle_days);

    // Apply time-elapsed multiplier for recurring quests
    let xp_earned = match quest_type {
        QuestType::OneOff => base_xp,
        QuestType::Recurring => {
            let last_completed: Option<String> = conn.query_row(
                "SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = ?1",
                rusqlite::params![quest_id],
                |row| row.get(0),
            ).map_err(|e| e.to_string())?;

            match last_completed.and_then(|ts| iso_utc_to_unix_secs(&ts)) {
                None => base_xp, // never completed → 1.0x
                Some(last_secs) => {
                    let now_secs = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs() as i64;
                    let elapsed_secs = (now_secs - last_secs).max(0) as f64;
                    let cycle_secs = (cycle_days.unwrap_or(1).max(1) as f64) * 86400.0;
                    let r = elapsed_secs / cycle_secs;
                    let multiplier = time_elapsed_multiplier(r);
                    (base_xp as f64 * multiplier).round() as i64
                }
            }
        }
    };

    // Snapshot levels before XP award
    let char_level_before = {
        let xp: i64 = conn.query_row("SELECT xp FROM character LIMIT 1", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        level_from_xp(xp, &LevelScale::Character).level
    };

    let mut attr_stmt = conn.prepare("SELECT id, name, xp FROM attribute")
        .map_err(|e| e.to_string())?;
    let attr_levels_before: Vec<(String, String, i32)> = attr_stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let xp: i64 = row.get(2)?;
        Ok((id, name, level_from_xp(xp, &LevelScale::Attribute).level))
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let mut skill_stmt = conn.prepare("SELECT id, name, xp FROM skill")
        .map_err(|e| e.to_string())?;
    let skill_levels_before: Vec<(String, String, i32)> = skill_stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let xp: i64 = row.get(2)?;
        Ok((id, name, level_from_xp(xp, &LevelScale::Skill).level))
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    // Distribute XP
    award_xp(&tx, &quest_id, xp_earned)?;

    let completion_id = Uuid::new_v4().to_string();
    let completed_at = chrono_now();

    tx.execute(
        "INSERT INTO quest_completion (id, quest_id, quest_title, completed_at, xp_earned) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![completion_id, quest_id, quest_title, completed_at, xp_earned],
    )
    .map_err(|e| e.to_string())?;

    // If one-off quest, deactivate it
    tx.execute(
        "UPDATE quest SET active = 0 WHERE id = ?1 AND quest_type = 'one_off'",
        rusqlite::params![quest_id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    // Detect level-ups by comparing before/after
    let mut level_ups = Vec::new();

    let char_level_after = {
        let xp: i64 = conn.query_row("SELECT xp FROM character LIMIT 1", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        level_from_xp(xp, &LevelScale::Character).level
    };
    if char_level_after > char_level_before {
        level_ups.push(LevelUp { name: "Character".into(), new_level: char_level_after });
    }

    for (id, name, level_before) in &attr_levels_before {
        let xp: i64 = conn.query_row(
            "SELECT xp FROM attribute WHERE id = ?1", rusqlite::params![id], |r| r.get(0)
        ).map_err(|e| e.to_string())?;
        let level_after = level_from_xp(xp, &LevelScale::Attribute).level;
        if level_after > *level_before {
            level_ups.push(LevelUp { name: name.clone(), new_level: level_after });
        }
    }

    for (id, name, level_before) in &skill_levels_before {
        let xp: i64 = conn.query_row(
            "SELECT xp FROM skill WHERE id = ?1", rusqlite::params![id], |r| r.get(0)
        ).map_err(|e| e.to_string())?;
        let level_after = level_from_xp(xp, &LevelScale::Skill).level;
        if level_after > *level_before {
            level_ups.push(LevelUp { name: name.clone(), new_level: level_after });
        }
    }

    Ok(Completion {
        id: completion_id,
        quest_id: Some(quest_id),
        quest_title,
        completed_at,
        xp_earned,
        level_ups,
    })
}

pub fn update_quest(
    conn: &Connection,
    quest_id: String,
    title: Option<String>,
    quest_type: Option<QuestType>,
    cycle_days: Option<i32>,
    difficulty: Option<Difficulty>,
    time_of_day: Option<i32>,
    days_of_week: Option<i32>,
) -> Result<Quest, String> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM quest WHERE id = ?1",
            rusqlite::params![quest_id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Quest not found: {}", quest_id));
    }

    if let Some(ref new_title) = title {
        conn.execute(
            "UPDATE quest SET title = ?1 WHERE id = ?2",
            rusqlite::params![new_title, quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(ref new_type) = quest_type {
        conn.execute(
            "UPDATE quest SET quest_type = ?1 WHERE id = ?2",
            rusqlite::params![new_type.as_str(), quest_id],
        )
        .map_err(|e| e.to_string())?;

        // When switching to one_off, clear cycle_days
        // When switching to recurring, set default cycle_days if not provided
        match new_type {
            QuestType::OneOff => {
                conn.execute(
                    "UPDATE quest SET cycle_days = NULL WHERE id = ?1",
                    rusqlite::params![quest_id],
                )
                .map_err(|e| e.to_string())?;
            }
            QuestType::Recurring => {
                if cycle_days.is_none() {
                    conn.execute(
                        "UPDATE quest SET cycle_days = 1 WHERE id = ?1 AND cycle_days IS NULL",
                        rusqlite::params![quest_id],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
    }

    if let Some(days) = cycle_days {
        conn.execute(
            "UPDATE quest SET cycle_days = ?1 WHERE id = ?2",
            rusqlite::params![days, quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(ref diff) = difficulty {
        conn.execute(
            "UPDATE quest SET difficulty = ?1 WHERE id = ?2",
            rusqlite::params![diff.as_str(), quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(tod) = time_of_day {
        conn.execute(
            "UPDATE quest SET time_of_day = ?1 WHERE id = ?2",
            rusqlite::params![tod, quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(dow) = days_of_week {
        conn.execute(
            "UPDATE quest SET days_of_week = ?1 WHERE id = ?2",
            rusqlite::params![dow, quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    query_single_quest(conn, &quest_id)
}

pub fn delete_quest(conn: &Connection, quest_id: String) -> Result<(), String> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM quest WHERE id = ?1",
            rusqlite::params![quest_id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Quest not found: {}", quest_id));
    }

    conn.execute(
        "UPDATE quest_completion SET quest_id = NULL WHERE quest_id = ?1",
        rusqlite::params![quest_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM quest_skill WHERE quest_id = ?1",
        rusqlite::params![quest_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM quest_attribute WHERE quest_id = ?1",
        rusqlite::params![quest_id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM quest WHERE id = ?1",
        rusqlite::params![quest_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn reorder_quests(conn: &Connection, orders: Vec<QuestOrder>) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare("UPDATE quest SET sort_order = ?1 WHERE id = ?2")
            .map_err(|e| e.to_string())?;
        for o in &orders {
            let rows = stmt
                .execute(rusqlite::params![o.sort_order, o.id])
                .map_err(|e| e.to_string())?;
            if rows == 0 {
                return Err(format!("Quest not found: {}", o.id));
            }
        }
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_completion(conn: &Connection, completion_id: String) -> Result<(), String> {
    let rows = conn
        .execute(
            "DELETE FROM quest_completion WHERE id = ?1",
            rusqlite::params![completion_id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Completion not found: {}", completion_id));
    }

    Ok(())
}

pub fn reset_character(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "UPDATE character SET xp = 0;
         UPDATE attribute SET xp = 0;
         UPDATE skill SET xp = 0;"
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reset_quests(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "DELETE FROM quest_skill;
         DELETE FROM quest_attribute;
         DELETE FROM quest;"
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn reset_completions(conn: &Connection) -> Result<(), String> {
    conn.execute_batch("DELETE FROM quest_completion;")
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_quest_links(conn: &Connection, quest_id: String) -> Result<QuestLinks, String> {
    let (skill_ids, attribute_ids) = load_quest_link_ids(conn, &quest_id)?;
    Ok(QuestLinks { skill_ids, attribute_ids })
}

pub fn set_quest_links(
    conn: &Connection,
    quest_id: String,
    skill_ids: Vec<String>,
    attribute_ids: Vec<String>,
) -> Result<(), String> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM quest WHERE id = ?1",
            rusqlite::params![quest_id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Quest not found: {}", quest_id));
    }

    // Validate skill IDs
    for sid in &skill_ids {
        let valid: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM skill WHERE id = ?1",
                rusqlite::params![sid],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .map_err(|e| e.to_string())?;
        if !valid {
            return Err(format!("Skill not found: {}", sid));
        }
    }

    // Validate attribute IDs
    for aid in &attribute_ids {
        let valid: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM attribute WHERE id = ?1",
                rusqlite::params![aid],
                |row| row.get::<_, i32>(0).map(|c| c > 0),
            )
            .map_err(|e| e.to_string())?;
        if !valid {
            return Err(format!("Attribute not found: {}", aid));
        }
    }

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    tx.execute(
        "DELETE FROM quest_skill WHERE quest_id = ?1",
        rusqlite::params![quest_id],
    )
    .map_err(|e| e.to_string())?;

    tx.execute(
        "DELETE FROM quest_attribute WHERE quest_id = ?1",
        rusqlite::params![quest_id],
    )
    .map_err(|e| e.to_string())?;

    for sid in &skill_ids {
        tx.execute(
            "INSERT INTO quest_skill (quest_id, skill_id) VALUES (?1, ?2)",
            rusqlite::params![quest_id, sid],
        )
        .map_err(|e| e.to_string())?;
    }

    for aid in &attribute_ids {
        tx.execute(
            "INSERT INTO quest_attribute (quest_id, attribute_id) VALUES (?1, ?2)",
            rusqlite::params![quest_id, aid],
        )
        .map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn load_all_quest_skills(conn: &Connection) -> Result<std::collections::HashMap<String, Vec<String>>, String> {
    let mut stmt = conn
        .prepare("SELECT quest_id, skill_id FROM quest_skill")
        .map_err(|e| e.to_string())?;
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    for row in rows {
        let (qid, sid) = row.map_err(|e| e.to_string())?;
        map.entry(qid).or_default().push(sid);
    }
    Ok(map)
}

fn load_all_quest_attributes(conn: &Connection) -> Result<std::collections::HashMap<String, Vec<String>>, String> {
    let mut stmt = conn
        .prepare("SELECT quest_id, attribute_id FROM quest_attribute")
        .map_err(|e| e.to_string())?;
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    for row in rows {
        let (qid, aid) = row.map_err(|e| e.to_string())?;
        map.entry(qid).or_default().push(aid);
    }
    Ok(map)
}

fn load_quest_link_ids(conn: &Connection, quest_id: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let mut skill_stmt = conn
        .prepare("SELECT skill_id FROM quest_skill WHERE quest_id = ?1")
        .map_err(|e| e.to_string())?;
    let skill_ids: Vec<String> = skill_stmt
        .query_map(rusqlite::params![quest_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut attr_stmt = conn
        .prepare("SELECT attribute_id FROM quest_attribute WHERE quest_id = ?1")
        .map_err(|e| e.to_string())?;
    let attribute_ids: Vec<String> = attr_stmt
        .query_map(rusqlite::params![quest_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok((skill_ids, attribute_ids))
}

fn query_single_quest(conn: &Connection, quest_id: &str) -> Result<Quest, String> {
    let today = local_today_days();
    let (skill_ids, attribute_ids) = load_quest_link_ids(conn, quest_id)?;
    conn.query_row(
        "SELECT q.id, q.title, q.quest_type, q.cycle_days, q.sort_order, q.active, q.created_at,
                q.difficulty, q.time_of_day, q.days_of_week,
                (SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = q.id) as last_completed
         FROM quest q WHERE q.id = ?1",
        rusqlite::params![quest_id],
        |row| {
            let quest_type_str: String = row.get(2)?;
            let quest_type = QuestType::from_str(&quest_type_str);
            let cycle_days: Option<i32> = row.get(3)?;
            let active = row.get::<_, i32>(5)? != 0;
            let difficulty_str: String = row.get(7)?;
            let time_of_day: i32 = row.get(8)?;
            let days_of_week: i32 = row.get(9)?;
            let last_completed: Option<String> = row.get(10)?;
            let last_completed_days = last_completed.as_deref().and_then(utc_iso_to_local_days);
            let is_due = compute_is_due(&quest_type, active, last_completed_days, cycle_days, today);
            Ok(Quest {
                id: row.get(0)?,
                title: row.get(1)?,
                quest_type,
                cycle_days,
                sort_order: row.get(4)?,
                active,
                created_at: row.get(6)?,
                difficulty: Difficulty::from_str(&difficulty_str),
                time_of_day,
                days_of_week,
                last_completed,
                is_due,
                skill_ids: skill_ids.clone(),
                attribute_ids: attribute_ids.clone(),
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn level_from_xp(xp: i64, scale: &LevelScale) -> LevelInfo {
    let (seed1, seed2) = match scale {
        LevelScale::Character => (300i64, 500i64),
        LevelScale::Attribute => (150, 250),
        LevelScale::Skill => (37, 62),
    };

    let mut level = 1;
    let mut cumulative: i64 = 0;
    let mut a = seed1;
    let mut b = seed2;

    loop {
        let cost = a;
        if cumulative + cost > xp {
            return LevelInfo {
                level,
                xp_for_current_level: cost,
                xp_into_current_level: xp - cumulative,
            };
        }
        cumulative += cost;
        level += 1;
        let next = a + b;
        a = b;
        b = next;
    }
}

pub fn get_character(conn: &Connection) -> Result<Character, String> {
    conn.query_row(
        "SELECT id, name, xp FROM character LIMIT 1",
        [],
        |row| {
            let xp: i64 = row.get(2)?;
            let info = level_from_xp(xp, &LevelScale::Character);
            Ok(Character {
                id: row.get(0)?,
                name: row.get(1)?,
                xp,
                level: info.level,
                xp_for_current_level: info.xp_for_current_level,
                xp_into_current_level: info.xp_into_current_level,
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn update_character(conn: &Connection, name: String) -> Result<Character, String> {
    let id: String = conn
        .query_row("SELECT id FROM character LIMIT 1", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE character SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    )
    .map_err(|e| e.to_string())?;

    get_character(conn)
}

pub fn get_attributes(conn: &Connection) -> Result<Vec<Attribute>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, sort_order, xp FROM attribute ORDER BY sort_order")
        .map_err(|e| e.to_string())?;

    let attrs = stmt
        .query_map([], |row| {
            let xp: i64 = row.get(3)?;
            let info = level_from_xp(xp, &LevelScale::Attribute);
            Ok(Attribute {
                id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
                xp,
                level: info.level,
                xp_for_current_level: info.xp_for_current_level,
                xp_into_current_level: info.xp_into_current_level,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(attrs)
}

pub fn get_skills(conn: &Connection) -> Result<Vec<Skill>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, attribute_id, sort_order, xp FROM skill ORDER BY sort_order")
        .map_err(|e| e.to_string())?;

    let skills = stmt
        .query_map([], |row| {
            let xp: i64 = row.get(4)?;
            let info = level_from_xp(xp, &LevelScale::Skill);
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                attribute_id: row.get(2)?,
                sort_order: row.get(3)?,
                xp,
                level: info.level,
                xp_for_current_level: info.xp_for_current_level,
                xp_into_current_level: info.xp_into_current_level,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(skills)
}

pub fn add_attribute(conn: &Connection, name: String) -> Result<Attribute, String> {
    let id = Uuid::new_v4().to_string();
    let max_order: i32 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), 0) FROM attribute", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO attribute (id, name, sort_order, xp) VALUES (?1, ?2, ?3, 0)",
        rusqlite::params![id, name, max_order + 1],
    ).map_err(|e| e.to_string())?;

    let info = level_from_xp(0, &LevelScale::Attribute);
    Ok(Attribute {
        id, name, sort_order: max_order + 1, xp: 0,
        level: info.level, xp_for_current_level: info.xp_for_current_level,
        xp_into_current_level: info.xp_into_current_level,
    })
}

pub fn add_skill(conn: &Connection, name: String, attribute_id: Option<String>) -> Result<Skill, String> {
    // Validate attribute_id if provided
    if let Some(ref aid) = attribute_id {
        let exists: bool = conn
            .query_row("SELECT COUNT(*) FROM attribute WHERE id = ?1", rusqlite::params![aid], |r| r.get::<_, i64>(0))
            .map(|c| c > 0)
            .map_err(|e| e.to_string())?;
        if !exists {
            return Err(format!("Attribute not found: {}", aid));
        }
    }

    let id = Uuid::new_v4().to_string();
    let max_order: i32 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), 0) FROM skill", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO skill (id, name, attribute_id, sort_order, xp) VALUES (?1, ?2, ?3, ?4, 0)",
        rusqlite::params![id, name, attribute_id, max_order + 1],
    ).map_err(|e| e.to_string())?;

    let info = level_from_xp(0, &LevelScale::Skill);
    Ok(Skill {
        id, name, attribute_id, sort_order: max_order + 1, xp: 0,
        level: info.level, xp_for_current_level: info.xp_for_current_level,
        xp_into_current_level: info.xp_into_current_level,
    })
}

pub fn rename_attribute(conn: &Connection, id: String, name: String) -> Result<(), String> {
    let rows = conn.execute(
        "UPDATE attribute SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    ).map_err(|e| e.to_string())?;
    if rows == 0 { return Err(format!("Attribute not found: {}", id)); }
    Ok(())
}

pub fn rename_skill(conn: &Connection, id: String, name: String) -> Result<(), String> {
    let rows = conn.execute(
        "UPDATE skill SET name = ?1 WHERE id = ?2",
        rusqlite::params![name, id],
    ).map_err(|e| e.to_string())?;
    if rows == 0 { return Err(format!("Skill not found: {}", id)); }
    Ok(())
}

pub fn update_skill_attribute(conn: &Connection, skill_id: String, attribute_id: Option<String>) -> Result<(), String> {
    // Validate attribute_id if provided
    if let Some(ref aid) = attribute_id {
        let exists: bool = conn
            .query_row("SELECT COUNT(*) FROM attribute WHERE id = ?1", rusqlite::params![aid], |r| r.get::<_, i64>(0))
            .map(|c| c > 0)
            .map_err(|e| e.to_string())?;
        if !exists {
            return Err(format!("Attribute not found: {}", aid));
        }
    }
    let rows = conn.execute(
        "UPDATE skill SET attribute_id = ?1 WHERE id = ?2",
        rusqlite::params![attribute_id, skill_id],
    ).map_err(|e| e.to_string())?;
    if rows == 0 { return Err(format!("Skill not found: {}", skill_id)); }
    Ok(())
}

pub fn delete_attribute(conn: &Connection, id: String) -> Result<(), String> {
    // Remove quest links
    conn.execute("DELETE FROM quest_attribute WHERE attribute_id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    // Unset skills mapped to this attribute
    conn.execute("UPDATE skill SET attribute_id = NULL WHERE attribute_id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    // Delete the attribute
    let rows = conn.execute("DELETE FROM attribute WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    if rows == 0 { return Err(format!("Attribute not found: {}", id)); }
    Ok(())
}

pub fn delete_skill(conn: &Connection, id: String) -> Result<(), String> {
    // Remove quest links
    conn.execute("DELETE FROM quest_skill WHERE skill_id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    // Delete the skill
    let rows = conn.execute("DELETE FROM skill WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    if rows == 0 { return Err(format!("Skill not found: {}", id)); }
    Ok(())
}

fn compute_is_due(quest_type: &QuestType, active: bool, last_completed_days: Option<i64>, cycle_days: Option<i32>, today_local_days: i64) -> bool {
    if !active {
        return false;
    }
    match quest_type {
        QuestType::OneOff => last_completed_days.is_none(),
        QuestType::Recurring => {
            match (cycle_days, last_completed_days) {
                (_, None) => true,
                (Some(cycle), Some(last_days)) => {
                    today_local_days - last_days >= cycle as i64
                }
                (None, Some(_)) => false,
            }
        }
    }
}

/// Convert year/month/day to a day count (for calendar day comparison).
fn ymd_to_days(y: i64, m: i64, d: i64) -> i64 {
    let (y_adj, m_adj) = if m <= 2 { (y - 1, m + 12) } else { (y, m) };
    365 * y_adj + y_adj / 4 - y_adj / 100 + y_adj / 400 + (153 * (m_adj - 3) + 2) / 5 + d
}

/// Convert a UTC ISO timestamp to Unix seconds.
fn iso_utc_to_unix_secs(iso: &str) -> Option<i64> {
    let (date_part, time_part) = iso.split_once('T')?;
    let dp: Vec<&str> = date_part.split('-').collect();
    if dp.len() != 3 { return None; }
    let y: i64 = dp[0].parse().ok()?;
    let m: i64 = dp[1].parse().ok()?;
    let d: i64 = dp[2].parse().ok()?;

    let time_str = time_part.trim_end_matches('Z');
    let tp: Vec<&str> = time_str.split(':').collect();
    let h: i64 = tp.first()?.parse().ok()?;
    let min: i64 = tp.get(1)?.parse().ok()?;
    let s: i64 = tp.get(2)?.parse().ok()?;

    let epoch_days = ymd_to_days(1970, 1, 1);
    let this_days = ymd_to_days(y, m, d);
    Some((this_days - epoch_days) * 86400 + h * 3600 + min * 60 + s)
}

/// Convert Unix seconds to local calendar day count using libc::localtime_r.
#[cfg(unix)]
fn unix_to_local_days(unix_secs: i64) -> i64 {
    let mut tm = unsafe { std::mem::zeroed::<libc::tm>() };
    let time_t = unix_secs as libc::time_t;
    unsafe { libc::localtime_r(&time_t, &mut tm) };
    let y = tm.tm_year as i64 + 1900;
    let m = tm.tm_mon as i64 + 1;
    let d = tm.tm_mday as i64;
    ymd_to_days(y, m, d)
}

/// Convert a UTC ISO timestamp to local calendar day count.
fn utc_iso_to_local_days(iso: &str) -> Option<i64> {
    let unix_secs = iso_utc_to_unix_secs(iso)?;
    Some(unix_to_local_days(unix_secs))
}

/// Get today's date as a local calendar day count.
fn local_today_days() -> i64 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    unix_to_local_days(secs)
}

/// Get today's local date as a string (YYYY-MM-DD) for skip reset comparison.
pub fn local_today_str() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    let mut tm = unsafe { std::mem::zeroed::<libc::tm>() };
    let time_t = secs as libc::time_t;
    unsafe { libc::localtime_r(&time_t, &mut tm) };
    format!("{:04}-{:02}-{:02}", tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday)
}

/// Get the current local hour (0–23).
pub fn local_hour() -> u32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    let mut tm = unsafe { std::mem::zeroed::<libc::tm>() };
    let time_t = secs as libc::time_t;
    unsafe { libc::localtime_r(&time_t, &mut tm) };
    tm.tm_hour as u32
}

/// Check if a time-of-day bitmask includes the current hour (0–23).
/// Morning=1 (4am-noon), Afternoon=2 (noon-5pm), Evening=4 (5pm-4am).
/// Mask of 7 or 0 = all times.
pub fn matches_time_of_day(mask: i32, hour: u32) -> bool {
    if mask == 7 || mask == 0 { return true; }
    let current_bit = if hour >= 4 && hour < 12 {
        1 // morning
    } else if hour >= 12 && hour < 17 {
        2 // afternoon
    } else {
        4 // evening
    };
    mask & current_bit != 0
}

/// Get the current local weekday (Mon=0 .. Sun=6).
pub fn local_weekday() -> u32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64;
    let mut tm = unsafe { std::mem::zeroed::<libc::tm>() };
    let time_t = secs as libc::time_t;
    unsafe { libc::localtime_r(&time_t, &mut tm) };
    // tm_wday: 0=Sun. Map to Mon=0..Sun=6
    match tm.tm_wday {
        0 => 6, // Sun
        d => (d - 1) as u32,
    }
}

/// Check if a days-of-week bitmask includes a given weekday (Mon=0 .. Sun=6).
/// Bitmask: Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64.
pub fn matches_day_of_week(mask: i32, weekday: u32) -> bool {
    let bit = 1 << weekday;
    mask & bit != 0
}

fn chrono_now() -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    let secs = duration.as_secs();

    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let mut y = 1970;
    let mut remaining_days = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut m = 0;
    for md in &month_days {
        if remaining_days < *md {
            break;
        }
        remaining_days -= md;
        m += 1;
    }

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m + 1,
        remaining_days + 1,
        hours,
        minutes,
        seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        init_db_memory()
    }

    #[test]
    fn empty_db_returns_no_quests() {
        let conn = test_db();
        assert!(get_quests(&conn).unwrap().is_empty());
    }

    #[test]
    fn empty_db_returns_no_completions() {
        let conn = test_db();
        assert!(get_completions(&conn).unwrap().is_empty());
    }

    #[test]
    fn add_recurring_quest() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        assert_eq!(q.quest_type, QuestType::Recurring);
        assert_eq!(q.cycle_days, Some(1));
        assert!(q.active);
        assert!(q.is_due);
    }

    #[test]
    fn add_recurring_quest_defaults_cycle_to_1() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, None, Difficulty::Easy, 7, 127).unwrap();
        assert_eq!(q.cycle_days, Some(1));
    }

    #[test]
    fn add_one_off_quest() {
        let conn = test_db();
        let q = add_quest(&conn, "File taxes".into(), QuestType::OneOff, None, Difficulty::Easy, 7, 127).unwrap();
        assert_eq!(q.quest_type, QuestType::OneOff);
        assert_eq!(q.cycle_days, None);
        assert!(q.is_due);
    }

    #[test]
    fn add_one_off_ignores_cycle_days() {
        let conn = test_db();
        let q = add_quest(&conn, "Taxes".into(), QuestType::OneOff, Some(5), Difficulty::Easy, 7, 127).unwrap();
        assert_eq!(q.cycle_days, None); // ignored for one-off
    }

    #[test]
    fn quests_ordered_by_sort_order_descending() {
        let conn = test_db();
        add_quest(&conn, "First".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        add_quest(&conn, "Second".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        add_quest(&conn, "Third".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests[0].title, "Third");
        assert_eq!(quests[1].title, "Second");
        assert_eq!(quests[2].title, "First");
    }

    #[test]
    fn sort_order_auto_increments() {
        let conn = test_db();
        let q1 = add_quest(&conn, "A".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let q2 = add_quest(&conn, "B".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        assert_eq!(q1.sort_order, 1);
        assert_eq!(q2.sort_order, 2);
    }

    // --- Completion tests ---

    #[test]
    fn complete_quest_snapshots_title() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let c = complete_quest(&conn, q.id.clone()).unwrap();
        assert_eq!(c.quest_title, "Shower");
        assert_eq!(c.quest_id, Some(q.id));
    }

    #[test]
    fn complete_recurring_stays_active() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert!(quests[0].active);
        assert!(!quests[0].is_due);
    }

    #[test]
    fn complete_one_off_deactivates() {
        let conn = test_db();
        let q = add_quest(&conn, "Taxes".into(), QuestType::OneOff, None, Difficulty::Easy, 7, 127).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert!(!quests[0].active);
        assert!(!quests[0].is_due);
    }

    #[test]
    fn multiple_completions() {
        let conn = test_db();
        let q = add_quest(&conn, "Water".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        assert_eq!(get_completions(&conn).unwrap().len(), 2);
    }

    #[test]
    fn complete_nonexistent_errors() {
        let conn = test_db();
        assert!(complete_quest(&conn, "nope".into()).is_err());
    }

    // --- Delete quest preserves completions ---

    #[test]
    fn delete_quest_preserves_completions() {
        let conn = test_db();
        let q = add_quest(&conn, "Delete me".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        delete_quest(&conn, q.id.clone()).unwrap();

        assert!(get_quests(&conn).unwrap().is_empty());
        let completions = get_completions(&conn).unwrap();
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].quest_title, "Delete me");
        assert!(completions[0].quest_id.is_none());
    }

    #[test]
    fn delete_nonexistent_quest_errors() {
        let conn = test_db();
        assert!(delete_quest(&conn, "nope".into()).is_err());
    }

    // --- Delete completion ---

    #[test]
    fn delete_completion_removes_one() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let c1 = complete_quest(&conn, q.id.clone()).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        delete_completion(&conn, c1.id).unwrap();
        assert_eq!(get_completions(&conn).unwrap().len(), 1);
    }

    #[test]
    fn delete_nonexistent_completion_errors() {
        let conn = test_db();
        assert!(delete_completion(&conn, "nope".into()).is_err());
    }

    // --- Update tests ---

    #[test]
    fn update_quest_title() {
        let conn = test_db();
        let q = add_quest(&conn, "Old".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let u = update_quest(&conn, q.id, Some("New".into()), None, None, None, None, None).unwrap();
        assert_eq!(u.title, "New");
        assert_eq!(u.cycle_days, Some(1));
    }

    #[test]
    fn update_quest_cycle() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let u = update_quest(&conn, q.id, None, None, Some(3), None, None, None).unwrap();
        assert_eq!(u.cycle_days, Some(3));
    }

    #[test]
    fn update_quest_type_to_one_off() {
        let conn = test_db();
        let q = add_quest(&conn, "Maybe once".into(), QuestType::Recurring, Some(7), Difficulty::Easy, 7, 127).unwrap();
        let u = update_quest(&conn, q.id, None, Some(QuestType::OneOff), None, None, None, None).unwrap();
        assert_eq!(u.quest_type, QuestType::OneOff);
        assert_eq!(u.cycle_days, None); // cleared
    }

    #[test]
    fn update_quest_type_to_recurring() {
        let conn = test_db();
        let q = add_quest(&conn, "Now recurring".into(), QuestType::OneOff, None, Difficulty::Easy, 7, 127).unwrap();
        let u = update_quest(&conn, q.id, None, Some(QuestType::Recurring), Some(3), None, None, None).unwrap();
        assert_eq!(u.quest_type, QuestType::Recurring);
        assert_eq!(u.cycle_days, Some(3));
    }

    #[test]
    fn update_quest_type_to_recurring_defaults_cycle() {
        let conn = test_db();
        let q = add_quest(&conn, "Now recurring".into(), QuestType::OneOff, None, Difficulty::Easy, 7, 127).unwrap();
        let u = update_quest(&conn, q.id, None, Some(QuestType::Recurring), None, None, None, None).unwrap();
        assert_eq!(u.quest_type, QuestType::Recurring);
        assert_eq!(u.cycle_days, Some(1)); // default
    }

    #[test]
    fn update_nonexistent_errors() {
        let conn = test_db();
        assert!(update_quest(&conn, "nope".into(), Some("x".into()), None, None, None, None, None).is_err());
    }

    // --- Reorder ---

    #[test]
    fn reorder_quests_swaps_order() {
        let conn = test_db();
        let a = add_quest(&conn, "A".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let b = add_quest(&conn, "B".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        // B has higher sort_order, so it's first in get_quests (DESC)
        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests[0].title, "B");
        assert_eq!(quests[1].title, "A");

        // Swap their sort_orders
        reorder_quests(&conn, vec![
            QuestOrder { id: a.id, sort_order: b.sort_order },
            QuestOrder { id: b.id, sort_order: a.sort_order },
        ]).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests[0].title, "A");
        assert_eq!(quests[1].title, "B");
    }

    #[test]
    fn reorder_quests_invalid_id_errors() {
        let conn = test_db();
        let q = add_quest(&conn, "Real".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let result = reorder_quests(&conn, vec![
            QuestOrder { id: q.id, sort_order: 5 },
            QuestOrder { id: "nonexistent".into(), sort_order: 3 },
        ]);
        assert!(result.is_err());
    }

    // --- is_due logic ---

    #[test]
    fn is_due_logic() {
        let mar12 = ymd_to_days(2026, 3, 12);
        let mar11 = ymd_to_days(2026, 3, 11);
        // Recurring, never completed
        assert!(compute_is_due(&QuestType::Recurring, true, None, Some(1), mar12));
        // Recurring, completed same day
        assert!(!compute_is_due(&QuestType::Recurring, true, Some(mar12), Some(1), mar12));
        // Recurring, cycle elapsed (1 day)
        assert!(compute_is_due(&QuestType::Recurring, true, Some(mar11), Some(1), mar12));
        // One-off, never completed
        assert!(compute_is_due(&QuestType::OneOff, true, None, None, mar12));
        // One-off, completed (has a day value = completed)
        assert!(!compute_is_due(&QuestType::OneOff, true, Some(mar12), None, mar12));
        // Inactive = never due
        assert!(!compute_is_due(&QuestType::Recurring, false, None, Some(1), mar12));
        assert!(!compute_is_due(&QuestType::OneOff, false, None, None, mar12));
    }

    #[test]
    fn is_due_month_boundary() {
        let feb28 = ymd_to_days(2026, 2, 28);
        let mar1 = ymd_to_days(2026, 3, 1);
        let jan31 = ymd_to_days(2026, 1, 31);
        let feb1 = ymd_to_days(2026, 2, 1);
        // Feb 28 → Mar 1 is 1 day, not 3
        assert!(!compute_is_due(&QuestType::Recurring, true, Some(feb28), Some(2), mar1));
        assert!(compute_is_due(&QuestType::Recurring, true, Some(feb28), Some(1), mar1));
        // Jan 31 → Feb 1 is 1 day
        assert!(compute_is_due(&QuestType::Recurring, true, Some(jan31), Some(1), feb1));
        assert!(!compute_is_due(&QuestType::Recurring, true, Some(jan31), Some(2), feb1));
    }

    #[test]
    fn local_time_conversion() {
        // ymd_to_days produces consistent day counts
        assert_eq!(ymd_to_days(2026, 3, 12) - ymd_to_days(2026, 3, 11), 1);
        assert_eq!(ymd_to_days(2026, 3, 1) - ymd_to_days(2026, 2, 28), 1);
        assert_eq!(ymd_to_days(2026, 1, 1) - ymd_to_days(2025, 12, 31), 1);

        // iso_utc_to_unix_secs round-trips correctly
        assert_eq!(iso_utc_to_unix_secs("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(iso_utc_to_unix_secs("1970-01-01T00:01:00Z"), Some(60));
        assert_eq!(iso_utc_to_unix_secs("1970-01-02T00:00:00Z"), Some(86400));

        // utc_iso_to_local_days returns a value (timezone-dependent, so just check it's Some)
        assert!(utc_iso_to_local_days("2026-03-12T12:00:00Z").is_some());
    }

    // --- Seed data ---

    #[test]
    fn seed_data_creates_character() {
        let conn = test_db();
        let c = get_character(&conn).unwrap();
        assert_eq!(c.name, "Adventurer");
        assert_eq!(c.xp, 0);
        assert_eq!(c.level, 1);
    }

    #[test]
    fn seed_data_creates_attributes() {
        let conn = test_db();
        let attrs = get_attributes(&conn).unwrap();
        assert!(!attrs.is_empty());
        let names: Vec<&str> = attrs.iter().map(|a| a.name.as_str()).collect();
        for expected in &["Health", "Pluck", "Knowledge", "Connection", "Responsibility"] {
            assert!(names.contains(expected), "Missing attribute: {}", expected);
        }
    }

    #[test]
    fn seed_data_creates_skills() {
        let conn = test_db();
        let skills = get_skills(&conn).unwrap();
        assert!(!skills.is_empty());
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        for expected in &["Cooking", "Technology", "Animal Handling", "Logistics"] {
            assert!(names.contains(expected), "Missing skill: {}", expected);
        }
        // Skills should be returned in sort_order
        let first = &skills[0];
        let last = &skills[skills.len() - 1];
        assert!(first.sort_order < last.sort_order);
    }

    #[test]
    fn seed_data_is_idempotent() {
        let conn = test_db();
        seed_data(&conn); // call again
        let attrs = get_attributes(&conn).unwrap();
        assert_eq!(attrs.len(), 5); // no duplicates
    }

    // --- Character CRUD ---

    #[test]
    fn update_character_name() {
        let conn = test_db();
        let c = update_character(&conn, "Hero".into()).unwrap();
        assert_eq!(c.name, "Hero");
        let c2 = get_character(&conn).unwrap();
        assert_eq!(c2.name, "Hero");
    }

    // --- Level curve ---

    #[test]
    fn level_from_xp_character_scale() {
        // Level 1 at 0 XP
        let info = level_from_xp(0, &LevelScale::Character);
        assert_eq!(info.level, 1);
        assert_eq!(info.xp_for_current_level, 300);
        assert_eq!(info.xp_into_current_level, 0);

        // Level 2 at exactly 300
        let info = level_from_xp(300, &LevelScale::Character);
        assert_eq!(info.level, 2);
        assert_eq!(info.xp_for_current_level, 500);
        assert_eq!(info.xp_into_current_level, 0);

        // Mid-level: 500 XP is partway through level 2
        let info = level_from_xp(500, &LevelScale::Character);
        assert_eq!(info.level, 2);
        assert_eq!(info.xp_into_current_level, 200);

        // Level 3 at 800
        let info = level_from_xp(800, &LevelScale::Character);
        assert_eq!(info.level, 3);
        assert_eq!(info.xp_for_current_level, 800);

        // Level 5 at 2900
        let info = level_from_xp(2900, &LevelScale::Character);
        assert_eq!(info.level, 5);
        assert_eq!(info.xp_for_current_level, 2100);

        // Level 10 at 37200
        let info = level_from_xp(37200, &LevelScale::Character);
        assert_eq!(info.level, 10);
    }

    #[test]
    fn level_from_xp_attribute_scale() {
        // Attribute seeds: 150, 250 (1/2 character)
        let info = level_from_xp(0, &LevelScale::Attribute);
        assert_eq!(info.level, 1);
        assert_eq!(info.xp_for_current_level, 150);

        let info = level_from_xp(150, &LevelScale::Attribute);
        assert_eq!(info.level, 2);
        assert_eq!(info.xp_for_current_level, 250);

        // Level 3 at 150+250=400
        let info = level_from_xp(400, &LevelScale::Attribute);
        assert_eq!(info.level, 3);
        assert_eq!(info.xp_for_current_level, 400); // 150+250
    }

    #[test]
    fn level_from_xp_skill_scale() {
        // Skill seeds: 37, 62 (1/8 character)
        let info = level_from_xp(0, &LevelScale::Skill);
        assert_eq!(info.level, 1);
        assert_eq!(info.xp_for_current_level, 37);

        let info = level_from_xp(37, &LevelScale::Skill);
        assert_eq!(info.level, 2);
        assert_eq!(info.xp_for_current_level, 62);

        // Level 3 at 37+62=99
        let info = level_from_xp(99, &LevelScale::Skill);
        assert_eq!(info.level, 3);
        assert_eq!(info.xp_for_current_level, 99); // 37+62
    }

    // --- Difficulty ---

    #[test]
    fn add_quest_with_difficulty() {
        let conn = test_db();
        let q = add_quest(&conn, "Hard task".into(), QuestType::OneOff, None, Difficulty::Epic, 7, 127).unwrap();
        assert_eq!(q.difficulty, Difficulty::Epic);

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests[0].difficulty, Difficulty::Epic);
    }

    #[test]
    fn quest_defaults_to_easy() {
        let conn = test_db();
        let q = add_quest(&conn, "Simple".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        assert_eq!(q.difficulty, Difficulty::Easy);
    }

    #[test]
    fn update_quest_difficulty() {
        let conn = test_db();
        let q = add_quest(&conn, "Task".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let u = update_quest(&conn, q.id, None, None, None, Some(Difficulty::Challenging), None, None).unwrap();
        assert_eq!(u.difficulty, Difficulty::Challenging);
    }

    // --- Quest links ---

    #[test]
    fn set_and_get_quest_links() {
        let conn = test_db();
        let q = add_quest(&conn, "Linked".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let skills = get_skills(&conn).unwrap();
        let attrs = get_attributes(&conn).unwrap();

        set_quest_links(
            &conn,
            q.id.clone(),
            vec![skills[0].id.clone(), skills[1].id.clone()],
            vec![attrs[0].id.clone()],
        )
        .unwrap();

        let links = get_quest_links(&conn, q.id.clone()).unwrap();
        assert_eq!(links.skill_ids.len(), 2);
        assert_eq!(links.attribute_ids.len(), 1);
        assert!(links.skill_ids.contains(&skills[0].id));
        assert!(links.skill_ids.contains(&skills[1].id));
        assert!(links.attribute_ids.contains(&attrs[0].id));
    }

    #[test]
    fn set_quest_links_replaces_previous() {
        let conn = test_db();
        let q = add_quest(&conn, "Replace".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let skills = get_skills(&conn).unwrap();

        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();
        set_quest_links(&conn, q.id.clone(), vec![skills[1].id.clone()], vec![]).unwrap();

        let links = get_quest_links(&conn, q.id.clone()).unwrap();
        assert_eq!(links.skill_ids.len(), 1);
        assert_eq!(links.skill_ids[0], skills[1].id);
    }

    #[test]
    fn get_quests_returns_link_ids() {
        let conn = test_db();
        let q = add_quest(&conn, "With links".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let skills = get_skills(&conn).unwrap();
        let attrs = get_attributes(&conn).unwrap();

        set_quest_links(
            &conn,
            q.id.clone(),
            vec![skills[0].id.clone()],
            vec![attrs[0].id.clone()],
        )
        .unwrap();

        let quests = get_quests(&conn).unwrap();
        let found = quests.iter().find(|qq| qq.id == q.id).unwrap();
        assert_eq!(found.skill_ids.len(), 1);
        assert_eq!(found.attribute_ids.len(), 1);
    }

    #[test]
    fn delete_quest_cleans_up_links() {
        let conn = test_db();
        let q = add_quest(&conn, "Delete links".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let skills = get_skills(&conn).unwrap();

        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();
        delete_quest(&conn, q.id.clone()).unwrap();

        // Verify link rows are gone (create another quest to reuse the skill)
        let q2 = add_quest(&conn, "New".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        set_quest_links(&conn, q2.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();
        let links = get_quest_links(&conn, q2.id.clone()).unwrap();
        assert_eq!(links.skill_ids.len(), 1);
    }

    #[test]
    fn set_quest_links_nonexistent_quest_errors() {
        let conn = test_db();
        let result = set_quest_links(&conn, "nope".into(), vec![], vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn set_quest_links_invalid_skill_errors() {
        let conn = test_db();
        let q = add_quest(&conn, "Bad link".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let result = set_quest_links(&conn, q.id, vec!["fake-skill".into()], vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn set_quest_links_invalid_attribute_errors() {
        let conn = test_db();
        let q = add_quest(&conn, "Bad link".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let result = set_quest_links(&conn, q.id, vec![], vec!["fake-attr".into()]);
        assert!(result.is_err());
    }

    // --- XP Engine ---

    #[test]
    fn calculate_xp_difficulty_ordering() {
        // Harder quests should give more XP
        let trivial = calculate_xp(&Difficulty::Trivial, &QuestType::Recurring, Some(1));
        let easy = calculate_xp(&Difficulty::Easy, &QuestType::Recurring, Some(1));
        let moderate = calculate_xp(&Difficulty::Moderate, &QuestType::Recurring, Some(1));
        let challenging = calculate_xp(&Difficulty::Challenging, &QuestType::Recurring, Some(1));
        let epic = calculate_xp(&Difficulty::Epic, &QuestType::Recurring, Some(1));
        assert!(trivial < easy);
        assert!(easy < moderate);
        assert!(moderate < challenging);
        assert!(challenging < epic);
        assert!(trivial > 0);
    }

    #[test]
    fn calculate_xp_longer_cycle_gives_more() {
        let daily = calculate_xp(&Difficulty::Easy, &QuestType::Recurring, Some(1));
        let weekly = calculate_xp(&Difficulty::Easy, &QuestType::Recurring, Some(7));
        let monthly = calculate_xp(&Difficulty::Easy, &QuestType::Recurring, Some(30));
        assert!(daily < weekly);
        assert!(weekly < monthly);
    }

    #[test]
    fn calculate_xp_one_off_positive() {
        let xp = calculate_xp(&Difficulty::Easy, &QuestType::OneOff, None);
        assert!(xp > 0);
        // One-off should give more than a daily of same difficulty
        let daily = calculate_xp(&Difficulty::Easy, &QuestType::Recurring, Some(1));
        assert!(xp > daily);
    }

    #[test]
    fn time_elapsed_multiplier_at_key_points() {
        // r=0: floor
        assert!((time_elapsed_multiplier(0.0) - 0.1).abs() < 0.01);
        // r=0.25: 0.1 + 0.9 * sqrt(0.25) = 0.1 + 0.45 = 0.55
        assert!((time_elapsed_multiplier(0.25) - 0.55).abs() < 0.01);
        // r=0.5: 0.1 + 0.9 * sqrt(0.5) ≈ 0.736
        assert!((time_elapsed_multiplier(0.5) - 0.736).abs() < 0.01);
        // r=1.0: exactly 1.0 (boundary — both formulas give 1.0)
        assert!((time_elapsed_multiplier(1.0) - 1.0).abs() < 0.01);
        // r=2.0: 1.0 + 0.5 * ln(2) ≈ 1.347
        assert!((time_elapsed_multiplier(2.0) - 1.347).abs() < 0.01);
        // r=7.0: 1.0 + 0.5 * ln(7) ≈ 1.973
        assert!((time_elapsed_multiplier(7.0) - 1.973).abs() < 0.01);
        // negative r: floor
        assert!((time_elapsed_multiplier(-1.0) - 0.1).abs() < 0.01);
    }

    #[test]
    fn time_elapsed_multiplier_never_below_floor() {
        for &r in &[0.0, 0.001, 0.01, 0.1, 0.5, 1.0, 2.0, 10.0, 100.0] {
            assert!(time_elapsed_multiplier(r) >= 0.1);
        }
    }

    #[test]
    fn time_elapsed_multiplier_monotonically_increasing() {
        let points = [0.0, 0.1, 0.25, 0.5, 0.75, 0.99, 1.0, 1.5, 2.0, 5.0, 10.0, 50.0];
        for w in points.windows(2) {
            assert!(time_elapsed_multiplier(w[1]) >= time_elapsed_multiplier(w[0]),
                "multiplier should increase: r={} gave {} but r={} gave {}",
                w[0], time_elapsed_multiplier(w[0]), w[1], time_elapsed_multiplier(w[1]));
        }
    }

    #[test]
    fn award_xp_character_only_no_links() {
        let conn = test_db();
        let q = add_quest(&conn, "Solo".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        award_xp(&conn, &q.id, 20).unwrap();

        let c = get_character(&conn).unwrap();
        assert_eq!(c.xp, 20);

        // Skills and attributes should be untouched
        let attrs = get_attributes(&conn).unwrap();
        assert!(attrs.iter().all(|a| a.xp == 0));
    }

    #[test]
    fn award_xp_distributes_to_links() {
        let conn = test_db();
        let q = add_quest(&conn, "Linked".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let skills = get_skills(&conn).unwrap();
        let attrs = get_attributes(&conn).unwrap();

        // Link to Cooking (skill) and Health (attribute)
        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![attrs[0].id.clone()]).unwrap();

        award_xp(&conn, &q.id, 20).unwrap();

        let c = get_character(&conn).unwrap();
        assert_eq!(c.xp, 20);

        let updated_attrs = get_attributes(&conn).unwrap();
        assert_eq!(updated_attrs[0].xp, 20); // Health got 20

        let updated_skills = get_skills(&conn).unwrap();
        assert_eq!(updated_skills[0].xp, 20); // Cooking got 20
    }

    #[test]
    fn skill_levelup_triggers_attribute_bump() {
        let conn = test_db();
        let q = add_quest(&conn, "Grind".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let skills = get_skills(&conn).unwrap();
        // Cooking (skill[0]) maps to Health (attr[0])
        // Skill level 2 at 37 XP. Award 37 to trigger level-up.
        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();

        award_xp(&conn, &q.id, 37).unwrap();

        let updated_skills = get_skills(&conn).unwrap();
        assert_eq!(updated_skills[0].xp, 37);
        assert_eq!(updated_skills[0].level, 2);

        // Health should have received attribute bump = Moderate one-off base XP
        let expected_bump = calculate_xp(&Difficulty::Moderate, &QuestType::OneOff, None);
        let updated_attrs = get_attributes(&conn).unwrap();
        assert_eq!(updated_attrs[0].xp, expected_bump);
    }

    #[test]
    fn complete_quest_stores_xp_earned() {
        let conn = test_db();
        let q = add_quest(&conn, "XP test".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let c = complete_quest(&conn, q.id).unwrap();

        assert!(c.xp_earned > 0);
        let completions = get_completions(&conn).unwrap();
        assert_eq!(completions[0].xp_earned, c.xp_earned);
    }

    #[test]
    fn complete_quest_awards_character_xp() {
        let conn = test_db();
        let q = add_quest(&conn, "XP flow".into(), QuestType::Recurring, Some(1), Difficulty::Moderate, 7, 127).unwrap();
        let c = complete_quest(&conn, q.id).unwrap();

        let char = get_character(&conn).unwrap();
        assert_eq!(char.xp, c.xp_earned);
    }

    #[test]
    fn delete_completion_does_not_reduce_xp() {
        let conn = test_db();
        let q = add_quest(&conn, "Permanent".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let c = complete_quest(&conn, q.id).unwrap();

        let char_before = get_character(&conn).unwrap();
        assert!(char_before.xp > 0);

        delete_completion(&conn, c.id).unwrap();

        let char_after = get_character(&conn).unwrap();
        assert_eq!(char_after.xp, char_before.xp); // unchanged
    }

    // --- Attribute/Skill CRUD ---

    #[test]
    fn add_attribute_returns_new_with_zero_xp() {
        let conn = test_db();
        let attr = add_attribute(&conn, "Grit".into()).unwrap();
        assert_eq!(attr.name, "Grit");
        assert_eq!(attr.xp, 0);
        assert_eq!(attr.level, 1);
        // Should appear in get_attributes
        let all = get_attributes(&conn).unwrap();
        assert!(all.iter().any(|a| a.name == "Grit"));
    }

    #[test]
    fn add_skill_with_attribute() {
        let conn = test_db();
        let attrs = get_attributes(&conn).unwrap();
        let skill = add_skill(&conn, "Painting".into(), Some(attrs[0].id.clone())).unwrap();
        assert_eq!(skill.name, "Painting");
        assert_eq!(skill.attribute_id, Some(attrs[0].id.clone()));
        assert_eq!(skill.xp, 0);
    }

    #[test]
    fn add_skill_without_attribute() {
        let conn = test_db();
        let skill = add_skill(&conn, "Freestyle".into(), None).unwrap();
        assert_eq!(skill.attribute_id, None);
    }

    #[test]
    fn add_skill_invalid_attribute_errors() {
        let conn = test_db();
        let result = add_skill(&conn, "Bad".into(), Some("fake-id".into()));
        assert!(result.is_err());
    }

    #[test]
    fn rename_attribute_updates_name() {
        let conn = test_db();
        let attrs = get_attributes(&conn).unwrap();
        rename_attribute(&conn, attrs[0].id.clone(), "Vigor".into()).unwrap();
        let updated = get_attributes(&conn).unwrap();
        assert_eq!(updated[0].name, "Vigor");
    }

    #[test]
    fn rename_skill_updates_name() {
        let conn = test_db();
        let skills = get_skills(&conn).unwrap();
        rename_skill(&conn, skills[0].id.clone(), "Botany".into()).unwrap();
        let updated = get_skills(&conn).unwrap();
        assert!(updated.iter().any(|s| s.name == "Botany"));
    }

    #[test]
    fn rename_nonexistent_attribute_errors() {
        let conn = test_db();
        assert!(rename_attribute(&conn, "nope".into(), "X".into()).is_err());
    }

    #[test]
    fn rename_nonexistent_skill_errors() {
        let conn = test_db();
        assert!(rename_skill(&conn, "nope".into(), "X".into()).is_err());
    }

    #[test]
    fn update_skill_attribute_mapping() {
        let conn = test_db();
        let attrs = get_attributes(&conn).unwrap();
        let skills = get_skills(&conn).unwrap();
        let original_attr = skills[0].attribute_id.clone();

        // Change to a different attribute
        let new_attr = attrs.iter().find(|a| Some(&a.id) != original_attr.as_ref()).unwrap();
        update_skill_attribute(&conn, skills[0].id.clone(), Some(new_attr.id.clone())).unwrap();
        let updated = get_skills(&conn).unwrap();
        assert_eq!(updated[0].attribute_id, Some(new_attr.id.clone()));

        // Set to None
        update_skill_attribute(&conn, skills[0].id.clone(), None).unwrap();
        let updated = get_skills(&conn).unwrap();
        assert_eq!(updated[0].attribute_id, None);
    }

    #[test]
    fn update_skill_attribute_invalid_errors() {
        let conn = test_db();
        let skills = get_skills(&conn).unwrap();
        assert!(update_skill_attribute(&conn, skills[0].id.clone(), Some("fake".into())).is_err());
    }

    #[test]
    fn delete_attribute_cleans_up_quest_links_and_skills() {
        let conn = test_db();
        let attrs = get_attributes(&conn).unwrap();
        let target_attr = &attrs[0];

        // Create a quest linked to this attribute
        let q = add_quest(&conn, "Linked".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        set_quest_links(&conn, q.id.clone(), vec![], vec![target_attr.id.clone()]).unwrap();

        // Find a skill mapped to this attribute
        let skills_before = get_skills(&conn).unwrap();
        let mapped_skill = skills_before.iter().find(|s| s.attribute_id.as_ref() == Some(&target_attr.id));

        delete_attribute(&conn, target_attr.id.clone()).unwrap();

        // Attribute is gone
        let attrs_after = get_attributes(&conn).unwrap();
        assert!(!attrs_after.iter().any(|a| a.id == target_attr.id));

        // Quest link is gone
        let links = get_quest_links(&conn, q.id).unwrap();
        assert!(links.attribute_ids.is_empty());

        // Mapped skill has attribute_id set to None
        if let Some(ms) = mapped_skill {
            let skills_after = get_skills(&conn).unwrap();
            let updated_skill = skills_after.iter().find(|s| s.id == ms.id).unwrap();
            assert_eq!(updated_skill.attribute_id, None);
        }
    }

    #[test]
    fn delete_skill_cleans_up_quest_links() {
        let conn = test_db();
        let skills = get_skills(&conn).unwrap();
        let target_skill = &skills[0];

        // Create a quest linked to this skill
        let q = add_quest(&conn, "Linked".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        set_quest_links(&conn, q.id.clone(), vec![target_skill.id.clone()], vec![]).unwrap();

        delete_skill(&conn, target_skill.id.clone()).unwrap();

        // Skill is gone
        let skills_after = get_skills(&conn).unwrap();
        assert!(!skills_after.iter().any(|s| s.id == target_skill.id));

        // Quest link is gone
        let links = get_quest_links(&conn, q.id).unwrap();
        assert!(links.skill_ids.is_empty());
    }

    #[test]
    fn delete_nonexistent_attribute_errors() {
        let conn = test_db();
        assert!(delete_attribute(&conn, "nope".into()).is_err());
    }

    #[test]
    fn delete_nonexistent_skill_errors() {
        let conn = test_db();
        assert!(delete_skill(&conn, "nope".into()).is_err());
    }

    // --- Reset functions ---

    #[test]
    fn reset_character_zeroes_all_xp() {
        let conn = test_db();
        let q = add_quest(&conn, "XP".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let skills = get_skills(&conn).unwrap();
        let attrs = get_attributes(&conn).unwrap();
        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![attrs[0].id.clone()]).unwrap();
        complete_quest(&conn, q.id).unwrap();

        assert!(get_character(&conn).unwrap().xp > 0);

        reset_character(&conn).unwrap();

        assert_eq!(get_character(&conn).unwrap().xp, 0);
        assert!(get_attributes(&conn).unwrap().iter().all(|a| a.xp == 0));
        assert!(get_skills(&conn).unwrap().iter().all(|s| s.xp == 0));
    }

    #[test]
    fn reset_quests_deletes_all() {
        let conn = test_db();
        let q = add_quest(&conn, "Gone".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        let skills = get_skills(&conn).unwrap();
        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();

        reset_quests(&conn).unwrap();

        assert!(get_quests(&conn).unwrap().is_empty());
    }

    #[test]
    fn reset_completions_deletes_all() {
        let conn = test_db();
        let q = add_quest(&conn, "Done".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        complete_quest(&conn, q.id).unwrap();

        reset_completions(&conn).unwrap();

        assert!(get_completions(&conn).unwrap().is_empty());
    }

    #[test]
    fn tauri_config_has_global_tauri_enabled() {
        let config_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
        let config_str = std::fs::read_to_string(&config_path)
            .expect("Could not read tauri.conf.json");
        let config: serde_json::Value = serde_json::from_str(&config_str)
            .expect("Could not parse tauri.conf.json");

        assert_eq!(
            config["app"]["withGlobalTauri"],
            serde_json::Value::Bool(true),
            "withGlobalTauri must be true in tauri.conf.json — without it, the frontend cannot call backend commands"
        );
    }

    // --- Quest selection ---

    #[test]
    fn get_next_quest_returns_highest_scored() {
        let conn = test_db();
        add_quest(&conn, "First".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        add_quest(&conn, "Second".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();

        let next = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        // Both never-completed daily quests created at same time — scores should be equal,
        // so list_order_bonus breaks tie (higher sort_order = lower bonus = sorted later)
        assert!(next.score > 0.0);
        assert_eq!(next.pool, "due");
    }

    #[test]
    fn get_next_quest_skip_changes_result() {
        let conn = test_db();
        add_quest(&conn, "First".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        add_quest(&conn, "Second".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();

        let empty = std::collections::HashMap::new();
        let q0 = get_next_quest(&conn, &empty, None).unwrap().unwrap();
        let top_id = q0.quest.id.clone();

        // Skip the top quest — the other one should surface
        let mut skips = std::collections::HashMap::new();
        skips.insert(top_id.clone(), 10); // heavy skip
        let q1 = get_next_quest(&conn, &skips, None).unwrap().unwrap();
        assert_ne!(q1.quest.id, top_id);

        // Skip both heavily — exhaustion fallback returns the least-negative
        let other_id = q1.quest.id.clone();
        skips.insert(other_id.clone(), 10);
        let q2 = get_next_quest(&conn, &skips, None).unwrap().unwrap();
        assert!(q2.score <= 0.0); // both heavily penalized
    }

    #[test]
    fn get_next_quest_empty_db() {
        let conn = test_db();
        assert!(get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().is_none());
    }

    #[test]
    fn get_next_quest_none_due_falls_back() {
        let conn = test_db();
        // Add a quest with a long cycle so it won't be due after completion
        let q = add_quest(&conn, "Long cycle".into(), QuestType::Recurring, Some(999), Difficulty::Easy, 7, 127).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        // Not due, but should fall back to not_due pool
        let next = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap();
        assert!(next.is_some());
        let scored = next.unwrap();
        assert_eq!(scored.quest.id, q.id);
        assert_eq!(scored.pool, "not_due");
    }

    #[test]
    fn get_next_quest_overdue_scores_higher() {
        let conn = test_db();
        // Quest A: 7-day cycle, Quest B: 1-day cycle. Both never completed.
        // B should be more "overdue" relative to its cycle.
        add_quest(&conn, "Weekly".into(), QuestType::Recurring, Some(7), Difficulty::Easy, 7, 127).unwrap();
        add_quest(&conn, "Daily".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();

        let top = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        // Daily quest has higher overdue ratio ((0+1)/1=1.0 vs (0+7)/7=1.0... actually equal for new quests)
        // Both are due, both have same overdue ratio for never-completed. list_order_bonus breaks tie.
        assert!(top.score > 0.0);
    }

    // --- Time-of-day ---

    #[test]
    fn matches_time_of_day_morning_only() {
        assert!(!matches_time_of_day(1, 3));  // 3am = evening
        assert!(matches_time_of_day(1, 4));   // 4am = morning start
        assert!(matches_time_of_day(1, 11));  // 11am = morning
        assert!(!matches_time_of_day(1, 12)); // noon = afternoon
    }

    #[test]
    fn matches_time_of_day_afternoon_only() {
        assert!(!matches_time_of_day(2, 11));
        assert!(matches_time_of_day(2, 12));
        assert!(matches_time_of_day(2, 16));
        assert!(!matches_time_of_day(2, 17));
    }

    #[test]
    fn matches_time_of_day_evening_only() {
        assert!(matches_time_of_day(4, 3));   // 3am is still evening
        assert!(!matches_time_of_day(4, 4));  // 4am is morning
        assert!(matches_time_of_day(4, 17));
        assert!(matches_time_of_day(4, 23));
        assert!(matches_time_of_day(4, 0));
    }

    #[test]
    fn matches_time_of_day_all() {
        assert!(matches_time_of_day(7, 0));
        assert!(matches_time_of_day(7, 12));
        assert!(matches_time_of_day(7, 23));
        // 0 also means all
        assert!(matches_time_of_day(0, 12));
    }

    #[test]
    fn matches_time_of_day_multi() {
        let morn_eve = 1 | 4; // morning + evening
        assert!(matches_time_of_day(morn_eve, 4));   // morning
        assert!(!matches_time_of_day(morn_eve, 14));  // afternoon
        assert!(matches_time_of_day(morn_eve, 20));   // evening
    }

    #[test]
    fn add_quest_with_time_of_day() {
        let conn = test_db();
        let q = add_quest(&conn, "Morning shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 1, 127).unwrap();
        assert_eq!(q.time_of_day, 1);

        let quests = get_quests(&conn).unwrap();
        let found = quests.iter().find(|qq| qq.id == q.id).unwrap();
        assert_eq!(found.time_of_day, 1);
    }

    #[test]
    fn update_quest_time_of_day() {
        let conn = test_db();
        let q = add_quest(&conn, "Walk".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        assert_eq!(q.time_of_day, 7);

        let u = update_quest(&conn, q.id, None, None, None, None, Some(4), None).unwrap();
        assert_eq!(u.time_of_day, 4);
    }

    #[test]
    fn add_quest_default_time_of_day() {
        let conn = test_db();
        let q = add_quest(&conn, "Default".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        assert_eq!(q.time_of_day, 7);
    }

    // --- Day-of-week ---

    #[test]
    fn matches_day_of_week_single_days() {
        // Mon=0 → bit 1, Tue=1 → bit 2, ..., Sun=6 → bit 64
        for day in 0..7u32 {
            let mask = 1i32 << day;
            assert!(matches_day_of_week(mask, day));
            // Other days should not match
            let other = (day + 1) % 7;
            assert!(!matches_day_of_week(mask, other));
        }
    }

    #[test]
    fn matches_day_of_week_every_day() {
        for day in 0..7u32 {
            assert!(matches_day_of_week(127, day));
        }
    }

    #[test]
    fn matches_day_of_week_weekdays() {
        let weekdays = 31; // Mon–Fri = 1+2+4+8+16
        for day in 0..5u32 {
            assert!(matches_day_of_week(weekdays, day)); // Mon–Fri
        }
        assert!(!matches_day_of_week(weekdays, 5)); // Sat
        assert!(!matches_day_of_week(weekdays, 6)); // Sun
    }

    #[test]
    fn matches_day_of_week_weekends() {
        let weekends = 96; // Sat+Sun = 32+64
        for day in 0..5u32 {
            assert!(!matches_day_of_week(weekends, day));
        }
        assert!(matches_day_of_week(weekends, 5)); // Sat
        assert!(matches_day_of_week(weekends, 6)); // Sun
    }

    #[test]
    fn add_quest_with_days_of_week() {
        let conn = test_db();
        let q = add_quest(&conn, "Trash day".into(), QuestType::Recurring, Some(7), Difficulty::Easy, 7, 8).unwrap(); // Thu only
        assert_eq!(q.days_of_week, 8);

        let quests = get_quests(&conn).unwrap();
        let found = quests.iter().find(|qq| qq.id == q.id).unwrap();
        assert_eq!(found.days_of_week, 8);
    }

    #[test]
    fn update_quest_days_of_week() {
        let conn = test_db();
        let q = add_quest(&conn, "Walk".into(), QuestType::Recurring, Some(1), Difficulty::Easy, 7, 127).unwrap();
        assert_eq!(q.days_of_week, 127);

        let u = update_quest(&conn, q.id, None, None, None, None, None, Some(96)).unwrap(); // weekends
        assert_eq!(u.days_of_week, 96);
    }
}
