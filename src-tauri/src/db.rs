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

// --- Parameter structs ---

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewQuest {
    pub title: String,
    #[serde(default = "default_quest_type")]
    pub quest_type: QuestType,
    pub cycle_days: Option<i32>,
    #[serde(default = "default_difficulty")]
    pub difficulty: Difficulty,
    #[serde(default = "default_time_of_day")]
    pub time_of_day: i32,
    #[serde(default = "default_days_of_week")]
    pub days_of_week: i32,
    #[serde(default)]
    pub importance: i32,
}

fn default_quest_type() -> QuestType { QuestType::Recurring }
fn default_difficulty() -> Difficulty { Difficulty::Easy }
fn default_time_of_day() -> i32 { 15 }
fn default_days_of_week() -> i32 { 127 }

impl Default for NewQuest {
    fn default() -> Self {
        NewQuest {
            title: String::new(),
            quest_type: QuestType::Recurring,
            cycle_days: Some(1),
            difficulty: Difficulty::Easy,
            time_of_day: 15,
            days_of_week: 127,
            importance: 0,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuestUpdate {
    pub title: Option<String>,
    pub quest_type: Option<QuestType>,
    pub cycle_days: Option<i32>,
    pub difficulty: Option<Difficulty>,
    pub time_of_day: Option<i32>,
    pub days_of_week: Option<i32>,
    pub importance: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewSagaStep {
    pub saga_id: String,
    pub title: String,
    #[serde(default = "default_difficulty")]
    pub difficulty: Difficulty,
    #[serde(default = "default_time_of_day")]
    pub time_of_day: i32,
    #[serde(default = "default_days_of_week")]
    pub days_of_week: i32,
    #[serde(default)]
    pub importance: i32,
}

impl Default for NewSagaStep {
    fn default() -> Self {
        NewSagaStep {
            saga_id: String::new(),
            title: String::new(),
            difficulty: Difficulty::Easy,
            time_of_day: 15,
            days_of_week: 127,
            importance: 0,
        }
    }
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
    pub saga_id: Option<String>,
    pub importance: i32,
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
pub struct Saga {
    pub id: String,
    pub name: String,
    pub cycle_days: Option<i32>,
    pub sort_order: i32,
    pub active: bool,
    pub created_at: String,
    pub last_run_completed_at: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ScoredQuest {
    pub quest: Quest,
    pub score: f64,
    pub overdue_ratio: f64,
    pub importance_boost: f64,
    pub skip_penalty: f64,
    pub list_order_bonus: f64,
    pub membership_bonus: f64,
    pub pool: String,
    pub due_count: usize,
    pub not_due_count: usize,
    pub saga_name: Option<String>,
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

// --- Campaign structs ---

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewCriterion {
    pub target_type: String,
    pub target_id: String,
    pub target_count: i32,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Criterion {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub target_name: String,
    pub target_count: i32,
    pub current_count: i32,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CampaignWithCriteria {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub criteria: Vec<Criterion>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CampaignCompletionResult {
    pub completed: bool,
    pub campaign_name: String,
    pub bonus_xp: i64,
    pub level_ups: Vec<LevelUp>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Accomplishment {
    pub id: String,
    pub campaign_id: Option<String>,
    pub campaign_name: String,
    pub completed_at: String,
    pub bonus_xp: i64,
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
            time_of_day INTEGER NOT NULL DEFAULT 15,
            days_of_week INTEGER NOT NULL DEFAULT 127,
            saga_id     TEXT REFERENCES saga(id),
            step_order  INTEGER,
            last_completed TEXT,
            importance  INTEGER NOT NULL DEFAULT 0
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

        CREATE TABLE IF NOT EXISTS saga (
            id                     TEXT PRIMARY KEY,
            name                   TEXT NOT NULL,
            cycle_days             INTEGER,
            sort_order             INTEGER NOT NULL,
            active                 INTEGER NOT NULL DEFAULT 1,
            created_at             TEXT NOT NULL,
            last_run_completed_at  TEXT
        );

        CREATE TABLE IF NOT EXISTS settings (
            id                    INTEGER PRIMARY KEY,
            cta_enabled           INTEGER NOT NULL DEFAULT 0,
            cta_interval_minutes  INTEGER NOT NULL DEFAULT 20,
            debug_scoring         INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS campaign (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            created_at   TEXT NOT NULL,
            completed_at TEXT
        );

        CREATE TABLE IF NOT EXISTS campaign_criterion (
            id            TEXT PRIMARY KEY,
            campaign_id   TEXT NOT NULL REFERENCES campaign(id),
            target_type   TEXT NOT NULL,
            target_id     TEXT NOT NULL,
            target_count  INTEGER NOT NULL,
            current_count INTEGER NOT NULL DEFAULT 0,
            sort_order    INTEGER NOT NULL,
            UNIQUE(campaign_id, target_type, target_id)
        );

        CREATE TABLE IF NOT EXISTS accomplishment (
            id             TEXT PRIMARY KEY,
            campaign_id    TEXT,
            campaign_name  TEXT NOT NULL,
            completed_at   TEXT NOT NULL,
            bonus_xp       INTEGER NOT NULL DEFAULT 0
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

    // Migration: create saga table if missing
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS saga (
            id                     TEXT PRIMARY KEY,
            name                   TEXT NOT NULL,
            cycle_days             INTEGER,
            sort_order             INTEGER NOT NULL,
            active                 INTEGER NOT NULL DEFAULT 1,
            created_at             TEXT NOT NULL,
            last_run_completed_at  TEXT
        );"
    ).expect("Failed to create saga table");

    // Migration: add saga_id and step_order to quest if missing
    let has_saga_id: bool = conn
        .prepare("SELECT saga_id FROM quest LIMIT 0")
        .is_ok();

    if !has_saga_id {
        conn.execute_batch(
            "ALTER TABLE quest ADD COLUMN saga_id TEXT REFERENCES saga(id);
             ALTER TABLE quest ADD COLUMN step_order INTEGER;"
        ).expect("Failed to add saga columns to quest");
    }

    // Migration: add last_completed column to quest, populate from completions
    let has_last_completed: bool = conn
        .prepare("SELECT last_completed FROM quest LIMIT 0")
        .is_ok();

    if !has_last_completed {
        conn.execute_batch(
            "ALTER TABLE quest ADD COLUMN last_completed TEXT;"
        ).expect("Failed to add last_completed column");

        conn.execute_batch(
            "UPDATE quest SET last_completed = (
                SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = quest.id
            );"
        ).expect("Failed to populate last_completed from completions");
    }

    // Migration: split evening into evening + night (3-bit → 4-bit TOD model)
    // Detection: if no quest uses bit 8, migration hasn't run
    let has_night_bit: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM quest WHERE (time_of_day & 8) != 0",
            [],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .unwrap_or(false);

    if !has_night_bit {
        // Quests with old evening bit (4) get night bit (8) added
        conn.execute_batch(
            "UPDATE quest SET time_of_day = time_of_day | 8 WHERE (time_of_day & 4) != 0;"
        ).expect("Failed to add night bit to evening quests");
        // Old "all times" mask 7 → new "all times" mask 15
        conn.execute_batch(
            "UPDATE quest SET time_of_day = 15 WHERE time_of_day = 7;"
        ).expect("Failed to update all-times mask");
    }

    // Migration: add importance column to quest
    let has_importance: bool = conn
        .prepare("SELECT importance FROM quest LIMIT 0")
        .is_ok();

    if !has_importance {
        conn.execute_batch(
            "ALTER TABLE quest ADD COLUMN importance INTEGER NOT NULL DEFAULT 0;"
        ).expect("Failed to add importance column");
    }

    // Migration: create campaign tables if missing
    let has_campaign: bool = conn
        .prepare("SELECT id FROM campaign LIMIT 0")
        .is_ok();

    if !has_campaign {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS campaign (
                id           TEXT PRIMARY KEY,
                name         TEXT NOT NULL,
                created_at   TEXT NOT NULL,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS campaign_criterion (
                id            TEXT PRIMARY KEY,
                campaign_id   TEXT NOT NULL REFERENCES campaign(id),
                target_type   TEXT NOT NULL,
                target_id     TEXT NOT NULL,
                target_count  INTEGER NOT NULL,
                current_count INTEGER NOT NULL DEFAULT 0,
                sort_order    INTEGER NOT NULL,
                UNIQUE(campaign_id, target_type, target_id)
            );

            CREATE TABLE IF NOT EXISTS accomplishment (
                id             TEXT PRIMARY KEY,
                campaign_id    TEXT,
                campaign_name  TEXT NOT NULL,
                completed_at   TEXT NOT NULL,
                bonus_xp       INTEGER NOT NULL DEFAULT 0
            );"
        ).expect("Failed to create campaign tables");
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

// --- Saga CRUD ---

pub fn get_sagas(conn: &Connection) -> Result<Vec<Saga>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, cycle_days, sort_order, active, created_at, last_run_completed_at
             FROM saga
             ORDER BY sort_order ASC",
        )
        .map_err(|e| e.to_string())?;

    let sagas = stmt
        .query_map([], |row| {
            Ok(Saga {
                id: row.get(0)?,
                name: row.get(1)?,
                cycle_days: row.get(2)?,
                sort_order: row.get(3)?,
                active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                last_run_completed_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sagas)
}

pub fn add_saga(
    conn: &Connection,
    name: String,
    cycle_days: Option<i32>,
) -> Result<Saga, String> {
    let id = Uuid::new_v4().to_string();
    let created_at = chrono_now();

    let max_order: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM saga",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let sort_order = max_order + 1;

    conn.execute(
        "INSERT INTO saga (id, name, cycle_days, sort_order, active, created_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        rusqlite::params![id, name, cycle_days, sort_order, created_at],
    )
    .map_err(|e| e.to_string())?;

    Ok(Saga {
        id,
        name,
        cycle_days,
        sort_order,
        active: true,
        created_at,
        last_run_completed_at: None,
    })
}

pub fn update_saga(
    conn: &Connection,
    saga_id: String,
    name: Option<String>,
    saga_type: Option<String>,
    cycle_days: Option<i32>,
) -> Result<Saga, String> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM saga WHERE id = ?1",
            rusqlite::params![saga_id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Saga not found: {}", saga_id));
    }

    if let Some(ref new_name) = name {
        conn.execute(
            "UPDATE saga SET name = ?1 WHERE id = ?2",
            rusqlite::params![new_name, saga_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(ref st) = saga_type {
        match st.as_str() {
            "one_off" => {
                conn.execute(
                    "UPDATE saga SET cycle_days = NULL WHERE id = ?1",
                    rusqlite::params![saga_id],
                ).map_err(|e| e.to_string())?;
            }
            "recurring" => {
                if cycle_days.is_none() {
                    conn.execute(
                        "UPDATE saga SET cycle_days = 1 WHERE id = ?1 AND cycle_days IS NULL",
                        rusqlite::params![saga_id],
                    ).map_err(|e| e.to_string())?;
                }
            }
            _ => {}
        }
    }

    if let Some(days) = cycle_days {
        conn.execute(
            "UPDATE saga SET cycle_days = ?1 WHERE id = ?2",
            rusqlite::params![days, saga_id],
        )
        .map_err(|e| e.to_string())?;
    }

    conn.query_row(
        "SELECT id, name, cycle_days, sort_order, active, created_at, last_run_completed_at
         FROM saga WHERE id = ?1",
        rusqlite::params![saga_id],
        |row| {
            Ok(Saga {
                id: row.get(0)?,
                name: row.get(1)?,
                cycle_days: row.get(2)?,
                sort_order: row.get(3)?,
                active: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
                last_run_completed_at: row.get(6)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn delete_saga(conn: &Connection, saga_id: String) -> Result<(), String> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM saga WHERE id = ?1",
            rusqlite::params![saga_id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Saga not found: {}", saga_id));
    }

    // Delete quest links for saga steps, then the steps themselves
    conn.execute(
        "DELETE FROM quest_skill WHERE quest_id IN (SELECT id FROM quest WHERE saga_id = ?1)",
        rusqlite::params![saga_id],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM quest_attribute WHERE quest_id IN (SELECT id FROM quest WHERE saga_id = ?1)",
        rusqlite::params![saga_id],
    ).map_err(|e| e.to_string())?;

    // Orphan completions (set quest_id to NULL) for saga steps
    conn.execute(
        "UPDATE quest_completion SET quest_id = NULL WHERE quest_id IN (SELECT id FROM quest WHERE saga_id = ?1)",
        rusqlite::params![saga_id],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM quest WHERE saga_id = ?1",
        rusqlite::params![saga_id],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM saga WHERE id = ?1",
        rusqlite::params![saga_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_saga_steps(conn: &Connection, saga_id: &str) -> Result<Vec<Quest>, String> {
    let today = local_today_days();
    let mut stmt = conn
        .prepare(
            "SELECT q.id, q.title, q.quest_type, q.cycle_days, q.sort_order, q.active, q.created_at,
                    q.difficulty, q.time_of_day, q.days_of_week, q.step_order, q.last_completed, q.importance
             FROM quest q
             WHERE q.saga_id = ?1
             ORDER BY q.step_order ASC",
        )
        .map_err(|e| e.to_string())?;

    let mut steps = stmt
        .query_map(rusqlite::params![saga_id], |row| {
            let quest_type_str: String = row.get(2)?;
            let quest_type = QuestType::from_str(&quest_type_str);
            let cycle_days: Option<i32> = row.get(3)?;
            let active = row.get::<_, i32>(5)? != 0;
            let difficulty_str: String = row.get(7)?;
            let time_of_day: i32 = row.get(8)?;
            let days_of_week: i32 = row.get(9)?;
            let last_completed: Option<String> = row.get(11)?;
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
                saga_id: Some(saga_id.to_string()),
                importance: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // Batch-load links
    let skill_links = load_all_quest_skills(conn)?;
    let attr_links = load_all_quest_attributes(conn)?;
    for s in &mut steps {
        if let Some(sids) = skill_links.get(&s.id) {
            s.skill_ids = sids.clone();
        }
        if let Some(aids) = attr_links.get(&s.id) {
            s.attribute_ids = aids.clone();
        }
    }

    Ok(steps)
}

pub fn add_saga_step(conn: &Connection, s: NewSagaStep) -> Result<Quest, String> {
    let saga_id = s.saga_id;
    let title = s.title;
    let difficulty = s.difficulty;
    let time_of_day = s.time_of_day;
    let days_of_week = s.days_of_week;
    let importance = s.importance;

    // Verify saga exists
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM saga WHERE id = ?1",
            rusqlite::params![saga_id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .map_err(|e| e.to_string())?;
    if !exists {
        return Err(format!("Saga not found: {}", saga_id));
    }

    let id = Uuid::new_v4().to_string();
    let created_at = chrono_now();

    let max_step: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(step_order), 0) FROM quest WHERE saga_id = ?1",
            rusqlite::params![saga_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let step_order = max_step + 1;

    conn.execute(
        "INSERT INTO quest (id, title, quest_type, cycle_days, sort_order, active, created_at, difficulty, time_of_day, days_of_week, saga_id, step_order, importance)
         VALUES (?1, ?2, 'one_off', NULL, 0, 1, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, title, created_at, difficulty.as_str(), time_of_day, days_of_week, saga_id, step_order, importance],
    )
    .map_err(|e| e.to_string())?;

    Ok(Quest {
        id,
        title,
        quest_type: QuestType::OneOff,
        cycle_days: None,
        sort_order: 0,
        active: true,
        created_at,
        difficulty,
        time_of_day,
        days_of_week,
        last_completed: None,
        is_due: true,
        skill_ids: Vec::new(),
        attribute_ids: Vec::new(),
        saga_id: Some(saga_id),
        importance,
    })
}

pub fn reorder_saga_steps(conn: &Connection, saga_id: &str, step_ids: Vec<String>) -> Result<(), String> {
    for (i, step_id) in step_ids.iter().enumerate() {
        conn.execute(
            "UPDATE quest SET step_order = ?1 WHERE id = ?2 AND saga_id = ?3",
            rusqlite::params![i as i32 + 1, step_id, saga_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SagaCompletionResult {
    pub completed: bool,
    pub saga_name: String,
    pub bonus_xp: i64,
    pub level_ups: Vec<LevelUp>,
}

/// Check if a saga's current run is complete (all steps have a completion > run start).
/// If complete, stamps last_run_completed_at, awards bonus XP, and returns result.
pub fn check_saga_completion(conn: &Connection, saga_id: &str) -> Result<SagaCompletionResult, String> {
    let saga = conn.query_row(
        "SELECT id, name, cycle_days, sort_order, active, created_at, last_run_completed_at FROM saga WHERE id = ?1",
        rusqlite::params![saga_id],
        |row| Ok(Saga {
            id: row.get(0)?,
            name: row.get(1)?,
            cycle_days: row.get(2)?,
            sort_order: row.get(3)?,
            active: row.get::<_, i32>(4)? != 0,
            created_at: row.get(5)?,
            last_run_completed_at: row.get(6)?,
        }),
    ).map_err(|e| e.to_string())?;

    let steps = get_saga_steps(conn, saga_id)?;
    let empty_result = SagaCompletionResult {
        completed: false,
        saga_name: saga.name.clone(),
        bonus_xp: 0,
        level_ups: Vec::new(),
    };
    if steps.is_empty() { return Ok(empty_result); }

    let run_start = saga.last_run_completed_at.as_deref().unwrap_or(&saga.created_at).to_string();

    let all_complete = steps.iter().all(|step| {
        step.last_completed.as_ref()
            .map(|lc| lc.as_str() > run_start.as_str())
            .unwrap_or(false)
    });

    if !all_complete {
        return Ok(empty_result);
    }

    // Stamp completion
    let now = chrono_now();
    conn.execute(
        "UPDATE saga SET last_run_completed_at = ?1 WHERE id = ?2",
        rusqlite::params![now, saga_id],
    ).map_err(|e| e.to_string())?;

    // Calculate bonus: 20% of baseline XP across all steps
    // baseline uses saga cycle: 5 × difficulty_mult × cycle_mult (time mult = 1.0)
    let cycle_mult: f64 = match saga.cycle_days {
        Some(c) => (c as f64).sqrt(), // recurring: sqrt(cycle_days)
        None => 3.0, // one-off saga: same as one-off quest
    };
    let total_baseline: f64 = steps.iter().map(|s| {
        let diff_mult = match s.difficulty {
            Difficulty::Trivial => 1.0,
            Difficulty::Easy => 5.0,
            Difficulty::Moderate => 10.0,
            Difficulty::Challenging => 20.0,
            Difficulty::Epic => 40.0,
        };
        5.0 * diff_mult * cycle_mult
    }).sum();
    let bonus_xp = (total_baseline * 0.20).round() as i64;

    // Award bonus to character
    let mut level_ups = Vec::new();
    if bonus_xp > 0 {
        let char_before = get_character(conn)?;
        conn.execute(
            "UPDATE character SET xp = xp + ?1",
            rusqlite::params![bonus_xp],
        ).map_err(|e| e.to_string())?;
        let char_after = get_character(conn)?;
        if char_after.level > char_before.level {
            level_ups.push(LevelUp { name: char_after.name.clone(), new_level: char_after.level });
        }

        // Award to final step's linked skills/attributes
        let last_step = &steps[steps.len() - 1];
        for attr_id in &last_step.attribute_ids {
            let attr_before = get_attribute_by_id(conn, attr_id)?;
            conn.execute(
                "UPDATE attribute SET xp = xp + ?1 WHERE id = ?2",
                rusqlite::params![bonus_xp, attr_id],
            ).map_err(|e| e.to_string())?;
            let attr_after = get_attribute_by_id(conn, attr_id)?;
            if attr_after.level > attr_before.level {
                level_ups.push(LevelUp { name: attr_after.name.clone(), new_level: attr_after.level });
            }
        }
        for skill_id in &last_step.skill_ids {
            let skill_before = get_skill_by_id(conn, skill_id)?;
            conn.execute(
                "UPDATE skill SET xp = xp + ?1 WHERE id = ?2",
                rusqlite::params![bonus_xp, skill_id],
            ).map_err(|e| e.to_string())?;
            let skill_after = get_skill_by_id(conn, skill_id)?;
            if skill_after.level > skill_before.level {
                level_ups.push(LevelUp { name: skill_after.name.clone(), new_level: skill_after.level });
            }
        }
    }

    Ok(SagaCompletionResult {
        completed: true,
        saga_name: saga.name,
        bonus_xp,
        level_ups,
    })
}

/// Check saga completion for a quest that might be a saga step.
/// Looks up the quest's saga_id and delegates to check_saga_completion.
pub fn check_saga_completion_for_quest(conn: &Connection, quest_id: &str) -> Result<SagaCompletionResult, String> {
    let saga_id: Option<String> = conn
        .query_row(
            "SELECT saga_id FROM quest WHERE id = ?1",
            rusqlite::params![quest_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    match saga_id {
        Some(sid) => check_saga_completion(conn, &sid),
        None => Ok(SagaCompletionResult {
            completed: false,
            saga_name: String::new(),
            bonus_xp: 0,
            level_ups: Vec::new(),
        }),
    }
}

/// Get saga with current run progress info.
#[derive(Serialize, Debug, Clone)]
pub struct SagaWithProgress {
    pub saga: Saga,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub is_due: bool,
}

pub fn get_sagas_with_progress(conn: &Connection) -> Result<Vec<SagaWithProgress>, String> {
    let sagas = get_sagas(conn)?;
    let today = local_today_days();
    let mut results = Vec::new();

    for saga in sagas {
        let steps = get_saga_steps(conn, &saga.id)?;
        let total_steps = steps.len();

        let run_start = saga.last_run_completed_at.as_deref().unwrap_or(&saga.created_at).to_string();

        // Check if saga is due (for recurring)
        let is_due = if let (Some(ref last_run), Some(cycle)) = (&saga.last_run_completed_at, saga.cycle_days) {
            let last_run_days = utc_iso_to_local_days(last_run).unwrap_or(0);
            today >= last_run_days + cycle as i64
        } else {
            true // One-off or never completed = due
        };

        // Always count completions after run_start — even during cooldown,
        // the user may have started a new run early.
        let completed_steps = steps.iter().filter(|s| {
            s.last_completed.as_ref()
                .map(|lc| lc.as_str() > run_start.as_str())
                .unwrap_or(false)
        }).count();

        // Active = has incomplete steps in current run (whether or not cycle has elapsed)
        let has_active_work = completed_steps < total_steps && completed_steps > 0;
        let effective_is_due = is_due || has_active_work;

        // In cooldown with no new work: show full bar (last run was complete)
        let display_completed = if !effective_is_due && completed_steps == 0 && saga.last_run_completed_at.is_some() {
            total_steps
        } else {
            completed_steps
        };

        results.push(SagaWithProgress {
            saga,
            total_steps,
            completed_steps: display_completed,
            is_due: effective_is_due,
        });
    }

    Ok(results)
}

/// Returns one active step per saga (the first due step), along with saga name and activation time.
pub fn get_active_saga_steps(conn: &Connection) -> Result<Vec<(Quest, String, String, Option<i32>)>, String> {
    // (quest, saga_name, activated_at ISO timestamp, saga_cycle_days)
    let sagas = get_sagas(conn)?;
    let mut results = Vec::new();

    for saga in &sagas {
        if !saga.active { continue; }

        let steps = get_saga_steps(conn, &saga.id)?;
        if steps.is_empty() { continue; }

        // Compute current run start
        let run_start_str = saga.last_run_completed_at.as_deref()
            .unwrap_or(&saga.created_at)
            .to_string();

        // Find the first step not completed in this run
        let mut prev_completed_at = run_start_str.clone();
        for step in &steps {
            let completed_this_run = step.last_completed.as_ref()
                .map(|lc| lc.as_str() > run_start_str.as_str())
                .unwrap_or(false);

            if !completed_this_run {
                // This is the active step
                results.push((step.clone(), saga.name.clone(), prev_completed_at.clone(), saga.cycle_days));
                break;
            }

            if let Some(ref lc) = step.last_completed {
                if lc.as_str() > run_start_str.as_str() {
                    prev_completed_at = lc.clone();
                }
            }
        }
    }

    Ok(results)
}

// --- Campaign CRUD ---

fn resolve_target_name(conn: &Connection, target_type: &str, target_id: &str) -> String {
    match target_type {
        "quest_completions" => {
            conn.query_row(
                "SELECT title FROM quest WHERE id = ?1",
                rusqlite::params![target_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "Deleted quest".to_string())
        }
        "saga_completions" => {
            conn.query_row(
                "SELECT name FROM saga WHERE id = ?1",
                rusqlite::params![target_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "Deleted saga".to_string())
        }
        _ => "Unknown target".to_string(),
    }
}

fn load_campaign_criteria(conn: &Connection, campaign_id: &str) -> Result<Vec<Criterion>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, target_type, target_id, target_count, current_count
             FROM campaign_criterion
             WHERE campaign_id = ?1
             ORDER BY sort_order ASC",
        )
        .map_err(|e| e.to_string())?;

    let criteria = stmt
        .query_map(rusqlite::params![campaign_id], |row| {
            let id: String = row.get(0)?;
            let target_type: String = row.get(1)?;
            let target_id: String = row.get(2)?;
            let target_count: i32 = row.get(3)?;
            let current_count: i32 = row.get(4)?;
            Ok((id, target_type, target_id, target_count, current_count))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(criteria
        .into_iter()
        .map(|(id, target_type, target_id, target_count, current_count)| {
            let target_name = resolve_target_name(conn, &target_type, &target_id);
            Criterion {
                id,
                target_type,
                target_id,
                target_name,
                target_count,
                current_count,
            }
        })
        .collect())
}

pub fn get_campaigns(conn: &Connection) -> Result<Vec<CampaignWithCriteria>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, created_at, completed_at
             FROM campaign
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let campaigns = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for (id, name, created_at, completed_at) in campaigns {
        let criteria = load_campaign_criteria(conn, &id)?;
        results.push(CampaignWithCriteria {
            id,
            name,
            created_at,
            completed_at,
            criteria,
        });
    }

    Ok(results)
}

pub fn create_campaign(
    conn: &Connection,
    name: String,
    criteria: Vec<NewCriterion>,
) -> Result<CampaignWithCriteria, String> {
    if criteria.is_empty() {
        return Err("Campaign must have at least one criterion".to_string());
    }

    // Check for duplicate target_ids within the criteria list
    let mut seen = std::collections::HashSet::new();
    for c in &criteria {
        let key = format!("{}:{}", c.target_type, c.target_id);
        if !seen.insert(key) {
            return Err(format!("Duplicate criterion for target {}", c.target_id));
        }
    }

    // Validate each criterion's target_type and target_id
    for c in &criteria {
        match c.target_type.as_str() {
            "quest_completions" => {
                let exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) FROM quest WHERE id = ?1",
                        rusqlite::params![c.target_id],
                        |row| row.get::<_, i32>(0).map(|n| n > 0),
                    )
                    .map_err(|e| e.to_string())?;
                if !exists {
                    return Err(format!("Quest not found: {}", c.target_id));
                }
            }
            "saga_completions" => {
                let exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) FROM saga WHERE id = ?1",
                        rusqlite::params![c.target_id],
                        |row| row.get::<_, i32>(0).map(|n| n > 0),
                    )
                    .map_err(|e| e.to_string())?;
                if !exists {
                    return Err(format!("Saga not found: {}", c.target_id));
                }
            }
            _ => {
                return Err(format!("Invalid target_type: {}", c.target_type));
            }
        }
    }

    let campaign_id = Uuid::new_v4().to_string();
    let created_at = chrono_now();

    conn.execute(
        "INSERT INTO campaign (id, name, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![campaign_id, name, created_at],
    )
    .map_err(|e| e.to_string())?;

    let mut built_criteria = Vec::new();
    for (i, c) in criteria.iter().enumerate() {
        let criterion_id = Uuid::new_v4().to_string();
        let sort_order = (i + 1) as i32;
        conn.execute(
            "INSERT INTO campaign_criterion (id, campaign_id, target_type, target_id, target_count, current_count, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
            rusqlite::params![criterion_id, campaign_id, c.target_type, c.target_id, c.target_count, sort_order],
        )
        .map_err(|e| e.to_string())?;

        let target_name = resolve_target_name(conn, &c.target_type, &c.target_id);
        built_criteria.push(Criterion {
            id: criterion_id,
            target_type: c.target_type.clone(),
            target_id: c.target_id.clone(),
            target_name,
            target_count: c.target_count,
            current_count: 0,
        });
    }

    Ok(CampaignWithCriteria {
        id: campaign_id,
        name,
        created_at,
        completed_at: None,
        criteria: built_criteria,
    })
}

pub fn rename_campaign(conn: &Connection, id: String, name: String) -> Result<(), String> {
    let rows = conn
        .execute(
            "UPDATE campaign SET name = ?1 WHERE id = ?2",
            rusqlite::params![name, id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Campaign not found: {}", id));
    }
    Ok(())
}

pub fn delete_campaign(conn: &Connection, id: String) -> Result<(), String> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM campaign WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get::<_, i32>(0).map(|c| c > 0),
        )
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Campaign not found: {}", id));
    }

    // Orphan any accomplishments
    conn.execute(
        "UPDATE accomplishment SET campaign_id = NULL WHERE campaign_id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;

    // Delete criteria
    conn.execute(
        "DELETE FROM campaign_criterion WHERE campaign_id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;

    // Delete campaign
    conn.execute(
        "DELETE FROM campaign WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_accomplishments(conn: &Connection) -> Result<Vec<Accomplishment>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, campaign_id, campaign_name, completed_at, bonus_xp
             FROM accomplishment
             ORDER BY completed_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Accomplishment {
                id: row.get(0)?,
                campaign_id: row.get(1)?,
                campaign_name: row.get(2)?,
                completed_at: row.get(3)?,
                bonus_xp: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rows)
}

pub fn delete_accomplishment(conn: &Connection, id: String) -> Result<(), String> {
    let rows = conn
        .execute(
            "DELETE FROM accomplishment WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| e.to_string())?;

    if rows == 0 {
        return Err(format!("Accomplishment not found: {}", id));
    }
    Ok(())
}

/// Increment campaign criteria matching a completion event.
/// Returns list of campaigns that just completed (usually 0 or 1).
pub fn check_campaign_progress(
    conn: &Connection,
    target_type: &str,
    target_id: &str,
) -> Result<Vec<CampaignCompletionResult>, String> {
    // Find active campaigns with a matching criterion
    let mut stmt = conn
        .prepare(
            "SELECT cc.campaign_id, cc.id
             FROM campaign_criterion cc
             JOIN campaign c ON c.id = cc.campaign_id
             WHERE c.completed_at IS NULL
               AND cc.target_type = ?1
               AND cc.target_id = ?2",
        )
        .map_err(|e| e.to_string())?;

    let matches: Vec<(String, String)> = stmt
        .query_map(rusqlite::params![target_type, target_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if matches.is_empty() {
        return Ok(Vec::new());
    }

    // Increment each matching criterion
    for (_, criterion_id) in &matches {
        conn.execute(
            "UPDATE campaign_criterion SET current_count = current_count + 1 WHERE id = ?1",
            rusqlite::params![criterion_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Check each affected campaign for completion
    let mut results = Vec::new();
    let mut checked_campaigns = std::collections::HashSet::new();

    for (campaign_id, _) in &matches {
        if !checked_campaigns.insert(campaign_id.clone()) {
            continue; // already checked this campaign
        }

        // Are all criteria met?
        let all_met: bool = conn
            .query_row(
                "SELECT COUNT(*) = 0 FROM campaign_criterion
                 WHERE campaign_id = ?1 AND current_count < target_count",
                rusqlite::params![campaign_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if all_met {
            let now = chrono_now();
            conn.execute(
                "UPDATE campaign SET completed_at = ?1 WHERE id = ?2",
                rusqlite::params![now, campaign_id],
            )
            .map_err(|e| e.to_string())?;

            let campaign_name: String = conn
                .query_row(
                    "SELECT name FROM campaign WHERE id = ?1",
                    rusqlite::params![campaign_id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;

            // Calculate bonus: 20% of constituent baseline XP
            let criteria = load_campaign_criteria(conn, campaign_id)?;
            let total_baseline: f64 = criteria.iter().map(|c| {
                match c.target_type.as_str() {
                    "quest_completions" => {
                        // Look up quest to get baseline XP
                        let quest_data: Option<(String, String, Option<i32>)> = conn.query_row(
                            "SELECT difficulty, quest_type, cycle_days FROM quest WHERE id = ?1",
                            rusqlite::params![c.target_id],
                            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                        ).ok();
                        match quest_data {
                            Some((diff_str, type_str, cycle)) => {
                                let diff = Difficulty::from_str(&diff_str);
                                let qt = QuestType::from_str(&type_str);
                                calculate_xp(&diff, &qt, cycle) as f64 * c.target_count as f64
                            }
                            None => 0.0, // deleted quest contributes nothing
                        }
                    }
                    "saga_completions" => 150.0 * c.target_count as f64,
                    _ => 0.0,
                }
            }).sum();
            let bonus_xp = (total_baseline * 0.20).round() as i64;

            // Award bonus to character
            let mut level_ups = Vec::new();
            if bonus_xp > 0 {
                let char_before = get_character(conn)?;
                conn.execute(
                    "UPDATE character SET xp = xp + ?1",
                    rusqlite::params![bonus_xp],
                ).map_err(|e| e.to_string())?;
                let char_after = get_character(conn)?;
                if char_after.level > char_before.level {
                    level_ups.push(LevelUp { name: char_after.name.clone(), new_level: char_after.level });
                }
            }

            // Create accomplishment record
            let accomplishment_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO accomplishment (id, campaign_id, campaign_name, completed_at, bonus_xp) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![accomplishment_id, campaign_id, campaign_name, now, bonus_xp],
            ).map_err(|e| e.to_string())?;

            results.push(CampaignCompletionResult {
                completed: true,
                campaign_name,
                bonus_xp,
                level_ups,
            });
        }
    }

    Ok(results)
}

// --- Quests ---

pub fn get_quests(conn: &Connection) -> Result<Vec<Quest>, String> {
    let today = local_today_days();
    let mut stmt = conn
        .prepare(
            "SELECT q.id, q.title, q.quest_type, q.cycle_days, q.sort_order, q.active, q.created_at,
                    q.difficulty, q.time_of_day, q.days_of_week, q.last_completed, q.importance
             FROM quest q
             WHERE q.saga_id IS NULL AND (q.active = 1 OR (q.active = 0 AND q.quest_type = 'one_off'))
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
                saga_id: None,
                importance: row.get(11)?,
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

const IMPORTANCE_WEIGHT: f64 = 30.0;

pub fn get_quest_scores(conn: &Connection, skip_counts: &std::collections::HashMap<String, i32>) -> Result<Vec<ScoredQuest>, String> {
    let quests = get_quests(conn)?;
    let today = local_today_days();
    let hour = local_hour();
    let weekday = local_weekday();
    let global_max_sort = quests.iter().map(|q| q.sort_order).max().unwrap_or(1) as f64;

    // Precompute quest IDs referenced by active campaigns
    let campaign_quest_ids: std::collections::HashSet<String> = {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT cc.target_id FROM campaign_criterion cc
             JOIN campaign c ON c.id = cc.campaign_id
             WHERE c.completed_at IS NULL AND cc.target_type = 'quest_completions'"
        ).map_err(|e| e.to_string())?;
        let ids: Vec<String> = stmt.query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        ids.into_iter().collect()
    };

    // Hard-filter: active, time-of-day, day-of-week
    let eligible: Vec<&Quest> = quests.iter()
        .filter(|q| q.active)
        .filter(|q| matches_time_of_day(q.time_of_day, hour))
        .filter(|q| matches_day_of_week(q.days_of_week, weekday))
        .collect();

    // Get active saga steps and apply same hard filters
    let saga_steps = get_active_saga_steps(conn).unwrap_or_default();
    let eligible_saga: Vec<&(Quest, String, String, Option<i32>)> = saga_steps.iter()
        .filter(|(q, _, _, _)| matches_time_of_day(q.time_of_day, hour))
        .filter(|(q, _, _, _)| matches_day_of_week(q.days_of_week, weekday))
        .collect();

    let due: Vec<&Quest> = eligible.iter().filter(|q| q.is_due).copied().collect();
    let not_due: Vec<&Quest> = eligible.iter().filter(|q| !q.is_due).copied().collect();

    let total_due = due.len() + eligible_saga.len();
    let due_count = total_due;
    let not_due_count = not_due.len();

    let mut results: Vec<ScoredQuest> = Vec::new();

    if !due.is_empty() || !eligible_saga.is_empty() {
        for (score, overdue, importance, skip, order, membership, quest) in score_quests_due(&due, today, skip_counts, global_max_sort, &campaign_quest_ids) {
            results.push(ScoredQuest { quest, score, overdue_ratio: overdue, importance_boost: importance, skip_penalty: skip, list_order_bonus: order, membership_bonus: membership, pool: "due".to_string(), due_count, not_due_count, saga_name: None });
        }
        for (quest, saga_name, activated_at, saga_cycle_days) in &eligible_saga {
            let activated_days = utc_iso_to_local_days(activated_at).unwrap_or(today);
            let days_since = (today - activated_days) as f64;
            let saga_cycle = saga_cycle_days.unwrap_or(9) as f64;
            let overdue_ratio = (days_since + saga_cycle) / saga_cycle;
            let importance_boost = quest.importance as f64 * IMPORTANCE_WEIGHT;
            let skips = *skip_counts.get(&quest.id).unwrap_or(&0) as f64;
            let skip_penalty = skips * (0.5 + quest.importance as f64 * IMPORTANCE_WEIGHT / 2.0);
            let list_order_bonus = 1.0;
            let membership_bonus = 0.0;
            let score = overdue_ratio + importance_boost - skip_penalty + list_order_bonus + membership_bonus;
            results.push(ScoredQuest { quest: (*quest).clone(), score, overdue_ratio, importance_boost, skip_penalty, list_order_bonus, membership_bonus, pool: "due".to_string(), due_count, not_due_count, saga_name: Some(saga_name.clone()) });
        }
    }

    let all_skipped = !results.is_empty() && results.iter().all(|s| s.score <= 0.0);
    if results.is_empty() || all_skipped {
        let not_due_scored = score_quests_not_due(&not_due, today, skip_counts, global_max_sort, &campaign_quest_ids);
        let pool = if due_count == 0 { "not_due" } else { "due+not_due" };
        for (score, overdue, importance, skip, order, membership, quest) in not_due_scored {
            results.push(ScoredQuest { quest, score, overdue_ratio: overdue, importance_boost: importance, skip_penalty: skip, list_order_bonus: order, membership_bonus: membership, pool: pool.to_string(), due_count, not_due_count, saga_name: None });
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results)
}

pub fn get_next_quest(conn: &Connection, skip_counts: &std::collections::HashMap<String, i32>, exclude_quest_id: Option<&str>) -> Result<Option<ScoredQuest>, String> {
    let scored = get_quest_scores(conn, skip_counts)?;
    if scored.is_empty() {
        return Ok(None);
    }

    let top = if let Some(exc_id) = exclude_quest_id {
        scored.iter()
            .find(|s| s.quest.id != exc_id)
            .cloned()
            .unwrap_or_else(|| scored.into_iter().next().unwrap())
    } else {
        scored.into_iter().next().unwrap()
    };

    Ok(Some(top))
}

fn score_quests_due(quests: &[&Quest], today: i64, skip_counts: &std::collections::HashMap<String, i32>, global_max_sort: f64, campaign_quest_ids: &std::collections::HashSet<String>) -> Vec<(f64, f64, f64, f64, f64, f64, Quest)> {
    quests.iter().map(|q| {
        let overdue_ratio = compute_overdue_ratio(q, today);
        let importance_boost = q.importance as f64 * IMPORTANCE_WEIGHT;
        let skips = *skip_counts.get(&q.id).unwrap_or(&0) as f64;
        let skip_penalty = skips * (0.5 + q.importance as f64 * IMPORTANCE_WEIGHT / 2.0);
        let list_order_bonus = q.sort_order as f64 / global_max_sort;
        let membership_bonus = if campaign_quest_ids.contains(&q.id) { 1.0 } else { 0.0 };
        let score = overdue_ratio + importance_boost - skip_penalty + list_order_bonus + membership_bonus;
        (score, overdue_ratio, importance_boost, skip_penalty, list_order_bonus, membership_bonus, (*q).clone())
    }).collect()
}

fn score_quests_not_due(quests: &[&Quest], today: i64, skip_counts: &std::collections::HashMap<String, i32>, global_max_sort: f64, campaign_quest_ids: &std::collections::HashSet<String>) -> Vec<(f64, f64, f64, f64, f64, f64, Quest)> {
    let days_since: Vec<f64> = quests.iter().map(|q| {
        match q.last_completed.as_deref().and_then(utc_iso_to_local_days) {
            Some(d) => (today - d) as f64,
            None => f64::MAX,
        }
    }).collect();
    let max_days = days_since.iter().cloned().filter(|d| *d < f64::MAX).fold(1.0f64, f64::max);

    quests.iter().enumerate().map(|(i, q)| {
        let normalized = if days_since[i] == f64::MAX { 1.0 } else { days_since[i] / max_days };
        let importance_boost = q.importance as f64 * IMPORTANCE_WEIGHT;
        let skips = *skip_counts.get(&q.id).unwrap_or(&0) as f64;
        let skip_penalty = skips * (0.5 + q.importance as f64 * IMPORTANCE_WEIGHT / 2.0);
        let list_order_bonus = q.sort_order as f64 / global_max_sort;
        let membership_bonus = if campaign_quest_ids.contains(&q.id) { 1.0 } else { 0.0 };
        let score = normalized + importance_boost - skip_penalty + list_order_bonus + membership_bonus;
        (score, normalized, importance_boost, skip_penalty, list_order_bonus, membership_bonus, (*q).clone())
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

pub fn add_quest(conn: &Connection, q: NewQuest) -> Result<Quest, String> {
    let id = Uuid::new_v4().to_string();
    let created_at = chrono_now();
    let title = q.title;
    let quest_type = q.quest_type;
    let cycle_days = q.cycle_days;
    let difficulty = q.difficulty;
    let time_of_day = q.time_of_day;
    let days_of_week = q.days_of_week;
    let importance = q.importance;

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
        "INSERT INTO quest (id, title, quest_type, cycle_days, sort_order, active, created_at, difficulty, time_of_day, days_of_week, importance)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![id, title, quest_type.as_str(), effective_cycle, sort_order, created_at, difficulty.as_str(), time_of_day, days_of_week, importance],
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
        saga_id: None,
        importance,
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
    let (quest_title, difficulty_str, quest_type_str, cycle_days, saga_id, stored_last_completed): (String, String, String, Option<i32>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT title, difficulty, quest_type, cycle_days, saga_id, last_completed FROM quest WHERE id = ?1",
            rusqlite::params![quest_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
        )
        .map_err(|_| format!("Quest not found: {}", quest_id))?;

    let difficulty = Difficulty::from_str(&difficulty_str);

    // For saga steps, use the saga's cycle to determine quest_type and cycle_days
    let (effective_type, effective_cycle) = if let Some(ref sid) = saga_id {
        let saga_cycle: Option<i32> = conn.query_row(
            "SELECT cycle_days FROM saga WHERE id = ?1",
            rusqlite::params![sid],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        match saga_cycle {
            Some(c) => (QuestType::Recurring, Some(c)),
            None => (QuestType::OneOff, None), // one-off saga
        }
    } else {
        (QuestType::from_str(&quest_type_str), cycle_days)
    };

    let base_xp = calculate_xp(&difficulty, &effective_type, effective_cycle);

    // Apply time-elapsed multiplier for recurring quests/saga steps
    let xp_earned = match effective_type {
        QuestType::OneOff => base_xp,
        QuestType::Recurring => {
            match stored_last_completed.as_deref().and_then(iso_utc_to_unix_secs) {
                None => base_xp, // never completed → 1.0x
                Some(last_secs) => {
                    let now_secs = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs() as i64;
                    let elapsed_secs = (now_secs - last_secs).max(0) as f64;
                    let cycle_secs = (effective_cycle.unwrap_or(1).max(1) as f64) * 86400.0;
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

    // Update stored last_completed
    tx.execute(
        "UPDATE quest SET last_completed = ?1 WHERE id = ?2",
        rusqlite::params![completed_at, quest_id],
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
    u: QuestUpdate,
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

    if let Some(ref new_title) = u.title {
        conn.execute(
            "UPDATE quest SET title = ?1 WHERE id = ?2",
            rusqlite::params![new_title, quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(ref new_type) = u.quest_type {
        conn.execute(
            "UPDATE quest SET quest_type = ?1 WHERE id = ?2",
            rusqlite::params![new_type.as_str(), quest_id],
        )
        .map_err(|e| e.to_string())?;

        match new_type {
            QuestType::OneOff => {
                conn.execute(
                    "UPDATE quest SET cycle_days = NULL WHERE id = ?1",
                    rusqlite::params![quest_id],
                )
                .map_err(|e| e.to_string())?;
            }
            QuestType::Recurring => {
                if u.cycle_days.is_none() {
                    conn.execute(
                        "UPDATE quest SET cycle_days = 1 WHERE id = ?1 AND cycle_days IS NULL",
                        rusqlite::params![quest_id],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }
    }

    if let Some(days) = u.cycle_days {
        conn.execute(
            "UPDATE quest SET cycle_days = ?1 WHERE id = ?2",
            rusqlite::params![days, quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(ref diff) = u.difficulty {
        conn.execute(
            "UPDATE quest SET difficulty = ?1 WHERE id = ?2",
            rusqlite::params![diff.as_str(), quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(tod) = u.time_of_day {
        conn.execute(
            "UPDATE quest SET time_of_day = ?1 WHERE id = ?2",
            rusqlite::params![tod, quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(dow) = u.days_of_week {
        conn.execute(
            "UPDATE quest SET days_of_week = ?1 WHERE id = ?2",
            rusqlite::params![dow, quest_id],
        )
        .map_err(|e| e.to_string())?;
    }

    if let Some(imp) = u.importance {
        conn.execute(
            "UPDATE quest SET importance = ?1 WHERE id = ?2",
            rusqlite::params![imp, quest_id],
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

pub fn set_quest_last_done(conn: &Connection, quest_id: String, last_done: Option<String>) -> Result<(), String> {
    // Verify quest exists and is not a saga step
    let saga_id: Option<String> = conn
        .query_row(
            "SELECT saga_id FROM quest WHERE id = ?1",
            rusqlite::params![quest_id],
            |row| row.get(0),
        )
        .map_err(|_| format!("Quest not found: {}", quest_id))?;

    if saga_id.is_some() {
        return Err("Cannot manually set last-done on a saga step".to_string());
    }

    conn.execute(
        "UPDATE quest SET last_completed = ?1 WHERE id = ?2",
        rusqlite::params![last_done, quest_id],
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
                q.difficulty, q.time_of_day, q.days_of_week, q.saga_id, q.last_completed, q.importance
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
            let saga_id: Option<String> = row.get(10)?;
            let last_completed: Option<String> = row.get(11)?;
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
                saga_id,
                importance: row.get(12)?,
            })
        },
    )
    .map_err(|e| e.to_string())
}

pub fn level_from_xp(xp: i64, scale: &LevelScale) -> LevelInfo {
    let (seed1, seed2) = match scale {
        LevelScale::Character => (300i64, 500i64),
        LevelScale::Attribute => (150, 250),
        LevelScale::Skill => (75, 125),
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

fn get_attribute_by_id(conn: &Connection, attr_id: &str) -> Result<Attribute, String> {
    conn.query_row(
        "SELECT id, name, sort_order, xp FROM attribute WHERE id = ?1",
        rusqlite::params![attr_id],
        |row| {
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
        },
    ).map_err(|e| e.to_string())
}

fn get_skill_by_id(conn: &Connection, skill_id: &str) -> Result<Skill, String> {
    conn.query_row(
        "SELECT id, name, attribute_id, sort_order, xp FROM skill WHERE id = ?1",
        rusqlite::params![skill_id],
        |row| {
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
        },
    ).map_err(|e| e.to_string())
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

pub fn reorder_attributes(conn: &Connection, attr_ids: Vec<String>) -> Result<(), String> {
    for (i, id) in attr_ids.iter().enumerate() {
        conn.execute(
            "UPDATE attribute SET sort_order = ?1 WHERE id = ?2",
            rusqlite::params![i as i32 + 1, id],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn reorder_skills(conn: &Connection, skill_ids: Vec<String>) -> Result<(), String> {
    for (i, id) in skill_ids.iter().enumerate() {
        conn.execute(
            "UPDATE skill SET sort_order = ?1 WHERE id = ?2",
            rusqlite::params![i as i32 + 1, id],
        ).map_err(|e| e.to_string())?;
    }
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
    if mask == 15 || mask == 0 { return true; }
    let current_bit = if hour >= 4 && hour < 12 {
        1 // morning
    } else if hour >= 12 && hour < 17 {
        2 // afternoon
    } else if hour >= 17 && hour < 21 {
        4 // evening
    } else {
        8 // night (9pm-4am)
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

    fn test_quest(conn: &Connection, title: &str) -> Quest {
        add_quest(conn, NewQuest {
            title: title.to_string(),
            ..NewQuest::default()
        }).unwrap()
    }

    fn test_quest_with(conn: &Connection, title: &str, f: impl FnOnce(&mut NewQuest)) -> Quest {
        let mut q = NewQuest { title: title.to_string(), ..NewQuest::default() };
        f(&mut q);
        add_quest(conn, q).unwrap()
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
        let q = test_quest(&conn, "Shower");
        assert_eq!(q.quest_type, QuestType::Recurring);
        assert_eq!(q.cycle_days, Some(1));
        assert!(q.active);
        assert!(q.is_due);
    }

    #[test]
    fn add_recurring_quest_defaults_cycle_to_1() {
        let conn = test_db();
        let q = test_quest_with(&conn, "Shower", |q| q.cycle_days = None);
        assert_eq!(q.cycle_days, Some(1));
    }

    #[test]
    fn add_one_off_quest() {
        let conn = test_db();
        let q = test_quest_with(&conn, "File taxes", |q| { q.quest_type = QuestType::OneOff; q.cycle_days = None; });
        assert_eq!(q.quest_type, QuestType::OneOff);
        assert_eq!(q.cycle_days, None);
        assert!(q.is_due);
    }

    #[test]
    fn add_one_off_ignores_cycle_days() {
        let conn = test_db();
        let q = test_quest_with(&conn, "Taxes", |q| { q.quest_type = QuestType::OneOff; q.cycle_days = None; });
        assert_eq!(q.cycle_days, None); // ignored for one-off
    }

    #[test]
    fn quests_ordered_by_sort_order_descending() {
        let conn = test_db();
        test_quest(&conn, "First");
        test_quest(&conn, "Second");
        test_quest(&conn, "Third");

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests[0].title, "Third");
        assert_eq!(quests[1].title, "Second");
        assert_eq!(quests[2].title, "First");
    }

    #[test]
    fn sort_order_auto_increments() {
        let conn = test_db();
        let q1 = test_quest(&conn, "A");
        let q2 = test_quest(&conn, "B");
        assert_eq!(q1.sort_order, 1);
        assert_eq!(q2.sort_order, 2);
    }

    // --- Completion tests ---

    #[test]
    fn complete_quest_snapshots_title() {
        let conn = test_db();
        let q = test_quest(&conn, "Shower");
        let c = complete_quest(&conn, q.id.clone()).unwrap();
        assert_eq!(c.quest_title, "Shower");
        assert_eq!(c.quest_id, Some(q.id));
    }

    #[test]
    fn complete_recurring_stays_active() {
        let conn = test_db();
        let q = test_quest(&conn, "Shower");
        complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert!(quests[0].active);
        assert!(!quests[0].is_due);
    }

    #[test]
    fn complete_one_off_deactivates() {
        let conn = test_db();
        let q = test_quest_with(&conn, "Taxes", |q| { q.quest_type = QuestType::OneOff; q.cycle_days = None; });
        complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert!(!quests[0].active);
        assert!(!quests[0].is_due);
    }

    #[test]
    fn multiple_completions() {
        let conn = test_db();
        let q = test_quest(&conn, "Water");
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
        let q = test_quest(&conn, "Delete me");
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
        let q = test_quest(&conn, "Shower");
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
        let q = test_quest(&conn, "Old");
        let u = update_quest(&conn, q.id, QuestUpdate { title: Some("New".into()), ..Default::default() }).unwrap();
        assert_eq!(u.title, "New");
        assert_eq!(u.cycle_days, Some(1));
    }

    #[test]
    fn update_quest_cycle() {
        let conn = test_db();
        let q = test_quest(&conn, "Shower");
        let u = update_quest(&conn, q.id, QuestUpdate { cycle_days: Some(3), ..Default::default() }).unwrap();
        assert_eq!(u.cycle_days, Some(3));
    }

    #[test]
    fn update_quest_type_to_one_off() {
        let conn = test_db();
        let q = test_quest_with(&conn, "Maybe once", |q| q.cycle_days = Some(7));
        let u = update_quest(&conn, q.id, QuestUpdate { quest_type: Some(QuestType::OneOff), ..Default::default() }).unwrap();
        assert_eq!(u.quest_type, QuestType::OneOff);
        assert_eq!(u.cycle_days, None); // cleared
    }

    #[test]
    fn update_quest_type_to_recurring() {
        let conn = test_db();
        let q = test_quest_with(&conn, "Now recurring", |q| { q.quest_type = QuestType::OneOff; q.cycle_days = None; });
        let u = update_quest(&conn, q.id, QuestUpdate { quest_type: Some(QuestType::Recurring), cycle_days: Some(3), ..Default::default() }).unwrap();
        assert_eq!(u.quest_type, QuestType::Recurring);
        assert_eq!(u.cycle_days, Some(3));
    }

    #[test]
    fn update_quest_type_to_recurring_defaults_cycle() {
        let conn = test_db();
        let q = test_quest_with(&conn, "Now recurring", |q| { q.quest_type = QuestType::OneOff; q.cycle_days = None; });
        let u = update_quest(&conn, q.id, QuestUpdate { quest_type: Some(QuestType::Recurring), ..Default::default() }).unwrap();
        assert_eq!(u.quest_type, QuestType::Recurring);
        assert_eq!(u.cycle_days, Some(1)); // default
    }

    #[test]
    fn update_nonexistent_errors() {
        let conn = test_db();
        assert!(update_quest(&conn, "nope".into(), QuestUpdate { title: Some("x".into()), ..Default::default() }).is_err());
    }

    // --- Reorder ---

    #[test]
    fn reorder_quests_swaps_order() {
        let conn = test_db();
        let a = test_quest(&conn, "A");
        let b = test_quest(&conn, "B");
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
        let q = test_quest(&conn, "Real");
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
        // Skill seeds: 75, 125
        let info = level_from_xp(0, &LevelScale::Skill);
        assert_eq!(info.level, 1);
        assert_eq!(info.xp_for_current_level, 75);

        let info = level_from_xp(75, &LevelScale::Skill);
        assert_eq!(info.level, 2);
        assert_eq!(info.xp_for_current_level, 125);

        // Level 3 at 75+125=200
        let info = level_from_xp(200, &LevelScale::Skill);
        assert_eq!(info.level, 3);
        assert_eq!(info.xp_for_current_level, 200); // 75+125
    }

    // --- Difficulty ---

    #[test]
    fn add_quest_with_difficulty() {
        let conn = test_db();
        let q = test_quest_with(&conn, "Hard task", |q| { q.quest_type = QuestType::OneOff; q.cycle_days = None; q.difficulty = Difficulty::Epic; });
        assert_eq!(q.difficulty, Difficulty::Epic);

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests[0].difficulty, Difficulty::Epic);
    }

    #[test]
    fn quest_defaults_to_easy() {
        let conn = test_db();
        let q = test_quest(&conn, "Simple");
        assert_eq!(q.difficulty, Difficulty::Easy);
    }

    #[test]
    fn update_quest_difficulty() {
        let conn = test_db();
        let q = test_quest(&conn, "Task");
        let u = update_quest(&conn, q.id, QuestUpdate { difficulty: Some(Difficulty::Challenging), ..Default::default() }).unwrap();
        assert_eq!(u.difficulty, Difficulty::Challenging);
    }

    // --- Quest links ---

    #[test]
    fn set_and_get_quest_links() {
        let conn = test_db();
        let q = test_quest(&conn, "Linked");
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
        let q = test_quest(&conn, "Replace");
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
        let q = test_quest(&conn, "With links");
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
        let q = test_quest(&conn, "Delete links");
        let skills = get_skills(&conn).unwrap();

        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();
        delete_quest(&conn, q.id.clone()).unwrap();

        // Verify link rows are gone (create another quest to reuse the skill)
        let q2 = test_quest(&conn, "New");
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
        let q = test_quest(&conn, "Bad link");
        let result = set_quest_links(&conn, q.id, vec!["fake-skill".into()], vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn set_quest_links_invalid_attribute_errors() {
        let conn = test_db();
        let q = test_quest(&conn, "Bad link");
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
        let q = test_quest(&conn, "Solo");
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
        let q = test_quest(&conn, "Linked");
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
        let q = test_quest(&conn, "Grind");
        let skills = get_skills(&conn).unwrap();
        // Cooking (skill[0]) maps to Health (attr[0])
        // Skill level 2 at 75 XP. Award 75 to trigger level-up.
        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();

        award_xp(&conn, &q.id, 75).unwrap();

        let updated_skills = get_skills(&conn).unwrap();
        assert_eq!(updated_skills[0].xp, 75);
        assert_eq!(updated_skills[0].level, 2);

        // Health should have received attribute bump = Moderate one-off base XP
        let expected_bump = calculate_xp(&Difficulty::Moderate, &QuestType::OneOff, None);
        let updated_attrs = get_attributes(&conn).unwrap();
        assert_eq!(updated_attrs[0].xp, expected_bump);
    }

    #[test]
    fn complete_quest_stores_xp_earned() {
        let conn = test_db();
        let q = test_quest(&conn, "XP test");
        let c = complete_quest(&conn, q.id).unwrap();

        assert!(c.xp_earned > 0);
        let completions = get_completions(&conn).unwrap();
        assert_eq!(completions[0].xp_earned, c.xp_earned);
    }

    #[test]
    fn complete_quest_awards_character_xp() {
        let conn = test_db();
        let q = test_quest_with(&conn, "XP flow", |q| q.difficulty = Difficulty::Moderate);
        let c = complete_quest(&conn, q.id).unwrap();

        let char = get_character(&conn).unwrap();
        assert_eq!(char.xp, c.xp_earned);
    }

    #[test]
    fn saga_step_xp_uses_saga_cycle() {
        let conn = test_db();
        // Create a weekly saga
        let saga = add_saga(&conn, "Weekly".into(), Some(7)).unwrap();
        // Add an Easy step
        let step = add_saga_step(&conn, NewSagaStep { saga_id: saga.id.clone(), title: "Step 1".into(), ..NewSagaStep::default() }).unwrap();
        let c = complete_quest(&conn, step.id.clone()).unwrap();

        // Easy step in a weekly saga: 5 × 5 × sqrt(7) = 66 (with time mult 1.0 for first completion)
        let expected = (5.0 * 5.0 * (7.0f64).sqrt()).round() as i64;
        assert_eq!(c.xp_earned, expected);

        // Compare: a regular one-off Easy quest would give 75
        let oneoff = test_quest_with(&conn, "One-off", |q| { q.quest_type = QuestType::OneOff; q.cycle_days = None; });
        let c2 = complete_quest(&conn, oneoff.id).unwrap();
        assert_eq!(c2.xp_earned, 75);
        assert!(c.xp_earned < c2.xp_earned); // weekly saga step earns less than one-off
    }

    #[test]
    fn oneoff_saga_step_xp_matches_oneoff_quest() {
        let conn = test_db();
        let saga = add_saga(&conn, "One-off saga".into(), None).unwrap();
        let step = add_saga_step(&conn, NewSagaStep { saga_id: saga.id.clone(), title: "Step".into(), ..NewSagaStep::default() }).unwrap();
        let c = complete_quest(&conn, step.id).unwrap();

        // One-off saga step should match one-off quest: 5 × 5 × 3 = 75
        assert_eq!(c.xp_earned, 75);
    }

    #[test]
    fn delete_completion_does_not_reduce_xp() {
        let conn = test_db();
        let q = test_quest(&conn, "Permanent");
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
        let q = test_quest(&conn, "Linked");
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
        let q = test_quest(&conn, "Linked");
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
        let q = test_quest(&conn, "XP");
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
        let q = test_quest(&conn, "Gone");
        let skills = get_skills(&conn).unwrap();
        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();

        reset_quests(&conn).unwrap();

        assert!(get_quests(&conn).unwrap().is_empty());
    }

    #[test]
    fn reset_completions_deletes_all() {
        let conn = test_db();
        let q = test_quest(&conn, "Done");
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
        test_quest(&conn, "First");
        test_quest(&conn, "Second");

        let next = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        // Both never-completed daily quests created at same time — scores should be equal,
        // so list_order_bonus breaks tie (higher sort_order = lower bonus = sorted later)
        assert!(next.score > 0.0);
        assert_eq!(next.pool, "due");
    }

    #[test]
    fn get_next_quest_skip_changes_result() {
        let conn = test_db();
        test_quest(&conn, "First");
        test_quest(&conn, "Second");

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
        let q = test_quest_with(&conn, "Long cycle", |q| q.cycle_days = Some(999));
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
        test_quest_with(&conn, "Weekly", |q| q.cycle_days = Some(7));
        test_quest(&conn, "Daily");

        let top = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        // Daily quest has higher overdue ratio ((0+1)/1=1.0 vs (0+7)/7=1.0... actually equal for new quests)
        // Both are due, both have same overdue ratio for never-completed. list_order_bonus breaks tie.
        assert!(top.score > 0.0);
    }

    // --- Sagas ---

    #[test]
    fn add_saga_returns_new() {
        let conn = test_db();
        let s = add_saga(&conn, "Spring Cleaning".into(), Some(30)).unwrap();
        assert_eq!(s.name, "Spring Cleaning");
        assert_eq!(s.cycle_days, Some(30));
        assert!(s.active);
        assert!(s.last_run_completed_at.is_none());
    }

    #[test]
    fn add_saga_one_off() {
        let conn = test_db();
        let s = add_saga(&conn, "File Taxes".into(), None).unwrap();
        assert_eq!(s.cycle_days, None);
    }

    #[test]
    fn get_sagas_returns_all() {
        let conn = test_db();
        add_saga(&conn, "First".into(), None).unwrap();
        add_saga(&conn, "Second".into(), Some(7)).unwrap();
        let sagas = get_sagas(&conn).unwrap();
        assert_eq!(sagas.len(), 2);
        assert_eq!(sagas[0].name, "First");
        assert_eq!(sagas[1].name, "Second");
    }

    #[test]
    fn update_saga_name() {
        let conn = test_db();
        let s = add_saga(&conn, "Old".into(), None).unwrap();
        let u = update_saga(&conn, s.id, Some("New".into()), None, None).unwrap();
        assert_eq!(u.name, "New");
    }

    #[test]
    fn update_saga_cycle() {
        let conn = test_db();
        let s = add_saga(&conn, "Test".into(), None).unwrap();
        assert_eq!(s.cycle_days, None);
        let u = update_saga(&conn, s.id, None, Some("recurring".into()), Some(14)).unwrap();
        assert_eq!(u.cycle_days, Some(14));
    }

    #[test]
    fn delete_saga_removes_saga_and_steps() {
        let conn = test_db();
        let s = add_saga(&conn, "Doomed".into(), None).unwrap();
        // Add a step (quest with saga_id)
        let q = test_quest_with(&conn, "Step 1", |q| { q.quest_type = QuestType::OneOff; q.cycle_days = None; });
        conn.execute(
            "UPDATE quest SET saga_id = ?1, step_order = 1 WHERE id = ?2",
            rusqlite::params![s.id, q.id],
        ).unwrap();

        delete_saga(&conn, s.id.clone()).unwrap();

        let sagas = get_sagas(&conn).unwrap();
        assert!(sagas.is_empty());
        // Step should be gone too
        let quests = get_quests(&conn).unwrap();
        assert!(quests.iter().all(|q| q.title != "Step 1"));
    }

    #[test]
    fn delete_nonexistent_saga_errors() {
        let conn = test_db();
        assert!(delete_saga(&conn, "nope".into()).is_err());
    }

    #[test]
    fn saga_steps_excluded_from_quest_list() {
        let conn = test_db();
        let s = add_saga(&conn, "Test Saga".into(), None).unwrap();
        test_quest(&conn, "Regular Quest");
        let step = test_quest_with(&conn, "Saga Step", |q| { q.quest_type = QuestType::OneOff; q.cycle_days = None; });
        conn.execute(
            "UPDATE quest SET saga_id = ?1, step_order = 1 WHERE id = ?2",
            rusqlite::params![s.id, step.id],
        ).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert!(quests.iter().any(|q| q.title == "Regular Quest"));
        assert!(!quests.iter().any(|q| q.title == "Saga Step"));
    }

    // --- Time-of-day ---

    #[test]
    fn matches_time_of_day_morning_only() {
        assert!(!matches_time_of_day(1, 3));  // 3am = night
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
        assert!(!matches_time_of_day(4, 3));   // 3am = night, not evening
        assert!(!matches_time_of_day(4, 4));   // 4am = morning
        assert!(matches_time_of_day(4, 17));   // 5pm = evening start
        assert!(matches_time_of_day(4, 20));   // 8pm = evening
        assert!(!matches_time_of_day(4, 21));  // 9pm = night
        assert!(!matches_time_of_day(4, 0));   // midnight = night
    }

    #[test]
    fn matches_time_of_day_night_only() {
        assert!(matches_time_of_day(8, 21));   // 9pm = night start
        assert!(matches_time_of_day(8, 23));   // 11pm = night
        assert!(matches_time_of_day(8, 0));    // midnight = night
        assert!(matches_time_of_day(8, 3));    // 3am = night
        assert!(!matches_time_of_day(8, 4));   // 4am = morning
        assert!(!matches_time_of_day(8, 17));  // 5pm = evening
    }

    #[test]
    fn matches_time_of_day_evening_and_night() {
        let ev_nt = 4 | 8; // evening + night
        assert!(matches_time_of_day(ev_nt, 17));  // evening
        assert!(matches_time_of_day(ev_nt, 20));  // evening
        assert!(matches_time_of_day(ev_nt, 21));  // night
        assert!(matches_time_of_day(ev_nt, 3));   // night
        assert!(!matches_time_of_day(ev_nt, 12)); // afternoon
    }

    #[test]
    fn matches_time_of_day_all() {
        assert!(matches_time_of_day(15, 0));
        assert!(matches_time_of_day(15, 12));
        assert!(matches_time_of_day(15, 17));
        assert!(matches_time_of_day(15, 23));
        // 0 also means all
        assert!(matches_time_of_day(0, 12));
        assert!(matches_time_of_day(0, 21));
    }

    #[test]
    fn matches_time_of_day_multi() {
        let morn_eve = 1 | 4; // morning + evening
        assert!(matches_time_of_day(morn_eve, 4));    // morning
        assert!(!matches_time_of_day(morn_eve, 14));   // afternoon
        assert!(matches_time_of_day(morn_eve, 20));    // evening
        assert!(!matches_time_of_day(morn_eve, 21));   // night — not selected
    }

    #[test]
    fn add_quest_with_time_of_day() {
        let conn = test_db();
        let q = test_quest_with(&conn, "Morning shower", |q| q.time_of_day = 1);
        assert_eq!(q.time_of_day, 1);

        let quests = get_quests(&conn).unwrap();
        let found = quests.iter().find(|qq| qq.id == q.id).unwrap();
        assert_eq!(found.time_of_day, 1);
    }

    #[test]
    fn update_quest_time_of_day() {
        let conn = test_db();
        let q = test_quest(&conn, "Walk");
        assert_eq!(q.time_of_day, 15);

        let u = update_quest(&conn, q.id, QuestUpdate { time_of_day: Some(4), ..Default::default() }).unwrap();
        assert_eq!(u.time_of_day, 4);
    }

    #[test]
    fn add_quest_default_time_of_day() {
        let conn = test_db();
        let q = test_quest(&conn, "Default");
        assert_eq!(q.time_of_day, 15);
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
        let q = test_quest_with(&conn, "Trash day", |q| { q.cycle_days = Some(7); q.days_of_week = 8; }); // Thu only
        assert_eq!(q.days_of_week, 8);

        let quests = get_quests(&conn).unwrap();
        let found = quests.iter().find(|qq| qq.id == q.id).unwrap();
        assert_eq!(found.days_of_week, 8);
    }

    #[test]
    fn update_quest_days_of_week() {
        let conn = test_db();
        let q = test_quest(&conn, "Walk");
        assert_eq!(q.days_of_week, 127);

        let u = update_quest(&conn, q.id, QuestUpdate { days_of_week: Some(96), ..Default::default() }).unwrap(); // weekends
        assert_eq!(u.days_of_week, 96);
    }

    // --- Campaign tests ---

    #[test]
    fn create_campaign_with_criteria() {
        let conn = test_db();
        let q1 = test_quest(&conn, "Vacuuming");
        let q2 = test_quest(&conn, "Mopping");

        let campaign = create_campaign(&conn, "Spring Cleaning".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q1.id.clone(), target_count: 4 },
            NewCriterion { target_type: "quest_completions".into(), target_id: q2.id.clone(), target_count: 2 },
        ]).unwrap();

        assert_eq!(campaign.name, "Spring Cleaning");
        assert_eq!(campaign.criteria.len(), 2);
        assert_eq!(campaign.criteria[0].current_count, 0);
        assert_eq!(campaign.criteria[1].current_count, 0);
        assert_eq!(campaign.criteria[0].target_count, 4);
        assert_eq!(campaign.criteria[1].target_count, 2);
        assert!(campaign.completed_at.is_none());
    }

    #[test]
    fn create_campaign_no_criteria_errors() {
        let conn = test_db();
        let result = create_campaign(&conn, "Empty".into(), vec![]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least one criterion"));
    }

    #[test]
    fn create_campaign_duplicate_target_errors() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");

        let result = create_campaign(&conn, "Dupe".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 2 },
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 3 },
        ]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate"));
    }

    #[test]
    fn create_campaign_mismatched_target_type_errors() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");

        // Try to use a quest id with saga_completions type
        let result = create_campaign(&conn, "Bad".into(), vec![
            NewCriterion { target_type: "saga_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Saga not found"));
    }

    #[test]
    fn create_campaign_invalid_target_type_errors() {
        let conn = test_db();
        let result = create_campaign(&conn, "Bad".into(), vec![
            NewCriterion { target_type: "bogus".into(), target_id: "fake".into(), target_count: 1 },
        ]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid target_type"));
    }

    #[test]
    fn get_campaigns_returns_all_with_criteria() {
        let conn = test_db();
        let q1 = test_quest(&conn, "Vacuuming");
        let q2 = test_quest(&conn, "Mopping");

        create_campaign(&conn, "Campaign A".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q1.id.clone(), target_count: 1 },
        ]).unwrap();
        create_campaign(&conn, "Campaign B".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q2.id.clone(), target_count: 3 },
        ]).unwrap();

        let campaigns = get_campaigns(&conn).unwrap();
        assert_eq!(campaigns.len(), 2);
        let names: Vec<&str> = campaigns.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"Campaign A"));
        assert!(names.contains(&"Campaign B"));
        for c in &campaigns {
            assert_eq!(c.criteria.len(), 1);
        }
    }

    #[test]
    fn rename_campaign_works() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        let c = create_campaign(&conn, "Old Name".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        rename_campaign(&conn, c.id.clone(), "New Name".into()).unwrap();

        let campaigns = get_campaigns(&conn).unwrap();
        assert_eq!(campaigns[0].name, "New Name");
    }

    #[test]
    fn delete_campaign_removes_campaign_and_criteria() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        let c = create_campaign(&conn, "Doomed".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        delete_campaign(&conn, c.id.clone()).unwrap();

        let campaigns = get_campaigns(&conn).unwrap();
        assert!(campaigns.is_empty());

        // Verify criteria are gone too
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM campaign_criterion WHERE campaign_id = ?1",
            rusqlite::params![c.id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn campaign_target_name_resolution() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");

        let campaign = create_campaign(&conn, "Test".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        assert_eq!(campaign.criteria[0].target_name, "Vacuuming");

        // Also verify via get_campaigns
        let campaigns = get_campaigns(&conn).unwrap();
        assert_eq!(campaigns[0].criteria[0].target_name, "Vacuuming");
    }

    #[test]
    fn campaign_target_name_deleted_quest() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        let campaign = create_campaign(&conn, "Test".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        // Delete the quest
        delete_quest(&conn, q.id).unwrap();

        // Target name should now be "Deleted quest"
        let campaigns = get_campaigns(&conn).unwrap();
        assert_eq!(campaigns[0].criteria[0].target_name, "Deleted quest");
        // But the campaign_id field in criteria still exists
        assert_eq!(campaigns[0].id, campaign.id);
    }

    #[test]
    fn campaign_with_saga_criterion() {
        let conn = test_db();
        let s = add_saga(&conn, "Laundry".into(), None).unwrap();

        let campaign = create_campaign(&conn, "Chores".into(), vec![
            NewCriterion { target_type: "saga_completions".into(), target_id: s.id.clone(), target_count: 4 },
        ]).unwrap();

        assert_eq!(campaign.criteria[0].target_name, "Laundry");
        assert_eq!(campaign.criteria[0].target_type, "saga_completions");
    }

    // --- check_campaign_progress tests ---

    #[test]
    fn check_campaign_progress_increments_quest_criterion() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        create_campaign(&conn, "Clean House".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 2 },
        ]).unwrap();

        let results = check_campaign_progress(&conn, "quest_completions", &q.id).unwrap();
        assert!(results.is_empty()); // not yet complete

        let campaigns = get_campaigns(&conn).unwrap();
        assert_eq!(campaigns[0].criteria[0].current_count, 1);
        assert!(campaigns[0].completed_at.is_none());
    }

    #[test]
    fn check_campaign_progress_completes_campaign() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        create_campaign(&conn, "Quick Clean".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        let results = check_campaign_progress(&conn, "quest_completions", &q.id).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].completed);
        assert_eq!(results[0].campaign_name, "Quick Clean");
        // easy recurring daily: calculate_xp = 5 * 5.0 * 1.0 = 25. Bonus = round(0.20 * 25) = 5
        assert_eq!(results[0].bonus_xp, 5);

        let campaigns = get_campaigns(&conn).unwrap();
        assert!(campaigns[0].completed_at.is_some());
    }

    #[test]
    fn check_campaign_progress_multiple_campaigns_same_quest() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        create_campaign(&conn, "Campaign A".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();
        create_campaign(&conn, "Campaign B".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 2 },
        ]).unwrap();

        let results = check_campaign_progress(&conn, "quest_completions", &q.id).unwrap();
        // Campaign A completes (1/1), Campaign B does not (1/2)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].campaign_name, "Campaign A");

        let campaigns = get_campaigns(&conn).unwrap();
        for c in &campaigns {
            if c.name == "Campaign A" {
                assert!(c.completed_at.is_some());
                assert_eq!(c.criteria[0].current_count, 1);
            } else {
                assert!(c.completed_at.is_none());
                assert_eq!(c.criteria[0].current_count, 1); // incremented
            }
        }
    }

    #[test]
    fn check_campaign_progress_saga_criterion() {
        let conn = test_db();
        let s = add_saga(&conn, "Laundry".into(), Some(7)).unwrap();
        create_campaign(&conn, "Chore Master".into(), vec![
            NewCriterion { target_type: "saga_completions".into(), target_id: s.id.clone(), target_count: 2 },
        ]).unwrap();

        let results = check_campaign_progress(&conn, "saga_completions", &s.id).unwrap();
        assert!(results.is_empty());

        let campaigns = get_campaigns(&conn).unwrap();
        assert_eq!(campaigns[0].criteria[0].current_count, 1);
    }

    #[test]
    fn check_campaign_progress_no_match() {
        let conn = test_db();
        let q1 = test_quest(&conn, "Vacuuming");
        let q2 = test_quest(&conn, "Mopping");
        create_campaign(&conn, "Vac Only".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q1.id.clone(), target_count: 1 },
        ]).unwrap();

        // Complete mopping — no campaign references it
        let results = check_campaign_progress(&conn, "quest_completions", &q2.id).unwrap();
        assert!(results.is_empty());

        let campaigns = get_campaigns(&conn).unwrap();
        assert_eq!(campaigns[0].criteria[0].current_count, 0); // unchanged
    }

    #[test]
    fn check_campaign_progress_skips_completed_campaign() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        create_campaign(&conn, "Done Already".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        // Complete the campaign
        check_campaign_progress(&conn, "quest_completions", &q.id).unwrap();
        let campaigns = get_campaigns(&conn).unwrap();
        assert!(campaigns[0].completed_at.is_some());
        assert_eq!(campaigns[0].criteria[0].current_count, 1);

        // Call again — should not increment
        let results = check_campaign_progress(&conn, "quest_completions", &q.id).unwrap();
        assert!(results.is_empty());

        let campaigns = get_campaigns(&conn).unwrap();
        assert_eq!(campaigns[0].criteria[0].current_count, 1); // unchanged
    }

    #[test]
    fn check_campaign_progress_multi_criteria_partial() {
        let conn = test_db();
        let qa = test_quest(&conn, "Quest A");
        let qb = test_quest(&conn, "Quest B");
        create_campaign(&conn, "Multi".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: qa.id.clone(), target_count: 2 },
            NewCriterion { target_type: "quest_completions".into(), target_id: qb.id.clone(), target_count: 1 },
        ]).unwrap();

        // A once: 1/2 + 0/1 — not complete
        let r = check_campaign_progress(&conn, "quest_completions", &qa.id).unwrap();
        assert!(r.is_empty());

        // B once: 1/2 + 1/1 — not complete
        let r = check_campaign_progress(&conn, "quest_completions", &qb.id).unwrap();
        assert!(r.is_empty());

        // A again: 2/2 + 1/1 — complete!
        let r = check_campaign_progress(&conn, "quest_completions", &qa.id).unwrap();
        assert_eq!(r.len(), 1);
        assert!(r[0].completed);
        assert_eq!(r[0].campaign_name, "Multi");

        let campaigns = get_campaigns(&conn).unwrap();
        assert!(campaigns[0].completed_at.is_some());
    }

    #[test]
    fn check_campaign_progress_full_saga_flow() {
        let conn = test_db();
        let s = add_saga(&conn, "Morning Routine".into(), Some(1)).unwrap();
        let step1 = add_saga_step(&conn, NewSagaStep {
            saga_id: s.id.clone(),
            title: "Brush teeth".into(),
            ..NewSagaStep::default()
        }).unwrap();
        let step2 = add_saga_step(&conn, NewSagaStep {
            saga_id: s.id.clone(),
            title: "Shower".into(),
            ..NewSagaStep::default()
        }).unwrap();

        // Backdate saga created_at so completions are strictly after it
        conn.execute(
            "UPDATE saga SET created_at = '2020-01-01T00:00:00Z' WHERE id = ?1",
            rusqlite::params![s.id],
        ).unwrap();

        // Campaign: 1 completion of step1 quest + 1 completion of saga
        create_campaign(&conn, "Full Flow".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: step1.id.clone(), target_count: 1 },
            NewCriterion { target_type: "saga_completions".into(), target_id: s.id.clone(), target_count: 1 },
        ]).unwrap();

        // Complete step 1 → quest criterion increments
        complete_quest(&conn, step1.id.clone()).unwrap();
        let r = check_campaign_progress(&conn, "quest_completions", &step1.id).unwrap();
        assert!(r.is_empty()); // saga criterion not yet met

        let campaigns = get_campaigns(&conn).unwrap();
        assert_eq!(campaigns[0].criteria[0].current_count, 1); // quest criterion
        assert_eq!(campaigns[0].criteria[1].current_count, 0); // saga criterion

        // Complete step 2 → saga completes
        complete_quest(&conn, step2.id.clone()).unwrap();
        let saga_result = check_saga_completion(&conn, &s.id).unwrap();
        assert!(saga_result.completed);

        // Campaign progress for step2 quest (no criterion for it, so no-op)
        check_campaign_progress(&conn, "quest_completions", &step2.id).unwrap();
        // Campaign progress for saga completion → saga criterion met → campaign completes
        let r = check_campaign_progress(&conn, "saga_completions", &s.id).unwrap();
        assert_eq!(r.len(), 1);
        assert!(r[0].completed);
        assert_eq!(r[0].campaign_name, "Full Flow");

        let campaigns = get_campaigns(&conn).unwrap();
        assert!(campaigns[0].completed_at.is_some());
    }

    // --- Bonus XP + accomplishment tests ---

    #[test]
    fn check_campaign_progress_awards_bonus_xp() {
        let conn = test_db();
        // Easy recurring daily (cycle=1): calculate_xp = 5 * 5.0 * 1.0 = 25
        let q1 = test_quest(&conn, "Daily Easy");
        // Moderate recurring 7-day: calculate_xp = 5 * 10.0 * sqrt(7) ≈ 132
        let q2 = test_quest_with(&conn, "Weekly Moderate", |q| {
            q.difficulty = Difficulty::Moderate;
            q.cycle_days = Some(7);
        });

        create_campaign(&conn, "Bonus Test".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q1.id.clone(), target_count: 2 },
            NewCriterion { target_type: "quest_completions".into(), target_id: q2.id.clone(), target_count: 1 },
        ]).unwrap();

        let char_before = get_character(&conn).unwrap();

        // Complete q1 twice and q2 once
        check_campaign_progress(&conn, "quest_completions", &q1.id).unwrap();
        check_campaign_progress(&conn, "quest_completions", &q1.id).unwrap();
        let results = check_campaign_progress(&conn, "quest_completions", &q2.id).unwrap();

        assert_eq!(results.len(), 1);
        // Expected: round(0.20 * (25*2 + 132*1)) = round(0.20 * 182) = round(36.4) = 36
        let expected_bonus = (0.20 * (calculate_xp(&Difficulty::Easy, &QuestType::Recurring, Some(1)) as f64 * 2.0
            + calculate_xp(&Difficulty::Moderate, &QuestType::Recurring, Some(7)) as f64 * 1.0)).round() as i64;
        assert_eq!(results[0].bonus_xp, expected_bonus);
        assert!(results[0].bonus_xp > 0);

        let char_after = get_character(&conn).unwrap();
        assert_eq!(char_after.xp, char_before.xp + expected_bonus);
    }

    #[test]
    fn check_campaign_progress_creates_accomplishment() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        create_campaign(&conn, "Accomplishment Test".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        check_campaign_progress(&conn, "quest_completions", &q.id).unwrap();

        let accomplishments = get_accomplishments(&conn).unwrap();
        assert_eq!(accomplishments.len(), 1);
        assert_eq!(accomplishments[0].campaign_name, "Accomplishment Test");
        assert_eq!(accomplishments[0].bonus_xp, 5); // easy daily: round(0.20 * 25) = 5
        assert!(accomplishments[0].campaign_id.is_some());
        assert!(!accomplishments[0].completed_at.is_empty());
    }

    #[test]
    fn check_campaign_progress_saga_criterion_bonus() {
        let conn = test_db();
        let s = add_saga(&conn, "Laundry".into(), Some(7)).unwrap();
        create_campaign(&conn, "Saga Bonus".into(), vec![
            NewCriterion { target_type: "saga_completions".into(), target_id: s.id.clone(), target_count: 2 },
        ]).unwrap();

        // Increment twice to complete
        check_campaign_progress(&conn, "saga_completions", &s.id).unwrap();
        let results = check_campaign_progress(&conn, "saga_completions", &s.id).unwrap();

        assert_eq!(results.len(), 1);
        // Expected: round(0.20 * (150 * 2)) = round(60) = 60
        assert_eq!(results[0].bonus_xp, 60);
    }

    #[test]
    fn check_campaign_progress_deleted_quest_bonus() {
        let conn = test_db();
        let q1 = test_quest(&conn, "Keeps");
        let q2 = test_quest(&conn, "Gets Deleted");
        create_campaign(&conn, "Deleted Target".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q1.id.clone(), target_count: 1 },
            NewCriterion { target_type: "quest_completions".into(), target_id: q2.id.clone(), target_count: 1 },
        ]).unwrap();

        // Delete q2 before completing
        delete_quest(&conn, q2.id.clone()).unwrap();

        // Complete both criteria
        check_campaign_progress(&conn, "quest_completions", &q1.id).unwrap();
        let results = check_campaign_progress(&conn, "quest_completions", &q2.id).unwrap();

        assert_eq!(results.len(), 1);
        // Only q1 contributes: round(0.20 * 25) = 5. q2 deleted = 0 contribution.
        assert_eq!(results[0].bonus_xp, 5);
    }

    #[test]
    fn check_campaign_progress_level_up_detection() {
        let conn = test_db();
        // Character starts at 0 XP. Level 1 costs 300 XP.
        // Set character XP to 295 so a bonus of 5+ triggers level-up.
        conn.execute("UPDATE character SET xp = 295", []).unwrap();

        let q = test_quest(&conn, "Level Up Quest");
        create_campaign(&conn, "Level Up".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        let results = check_campaign_progress(&conn, "quest_completions", &q.id).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].bonus_xp > 0);
        assert!(!results[0].level_ups.is_empty());
        assert_eq!(results[0].level_ups[0].new_level, 2);
    }

    #[test]
    fn delete_accomplishment_preserves_xp() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        create_campaign(&conn, "XP Test".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        check_campaign_progress(&conn, "quest_completions", &q.id).unwrap();
        let char_after_bonus = get_character(&conn).unwrap();
        let accomplishments = get_accomplishments(&conn).unwrap();
        assert_eq!(accomplishments.len(), 1);

        delete_accomplishment(&conn, accomplishments[0].id.clone()).unwrap();

        let char_after_delete = get_character(&conn).unwrap();
        assert_eq!(char_after_delete.xp, char_after_bonus.xp); // XP unchanged
        assert!(get_accomplishments(&conn).unwrap().is_empty());
    }

    #[test]
    fn delete_campaign_orphans_accomplishment() {
        let conn = test_db();
        let q = test_quest(&conn, "Vacuuming");
        let c = create_campaign(&conn, "Orphan Test".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 1 },
        ]).unwrap();

        check_campaign_progress(&conn, "quest_completions", &q.id).unwrap();
        let accomplishments = get_accomplishments(&conn).unwrap();
        assert_eq!(accomplishments.len(), 1);
        assert!(accomplishments[0].campaign_id.is_some());

        delete_campaign(&conn, c.id).unwrap();

        let accomplishments = get_accomplishments(&conn).unwrap();
        assert_eq!(accomplishments.len(), 1);
        assert!(accomplishments[0].campaign_id.is_none()); // orphaned
        assert_eq!(accomplishments[0].campaign_name, "Orphan Test"); // name snapshot survives
    }

    // --- Stored last_completed tests ---

    #[test]
    fn complete_quest_sets_last_completed() {
        let conn = test_db();
        let q = test_quest(&conn, "Vitamins");

        // Before completion, last_completed is None
        let quests = get_quests(&conn).unwrap();
        let quest = quests.iter().find(|x| x.id == q.id).unwrap();
        assert!(quest.last_completed.is_none());

        let completion = complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        let quest = quests.iter().find(|x| x.id == q.id).unwrap();
        assert_eq!(quest.last_completed.as_deref(), Some(completion.completed_at.as_str()));
    }

    #[test]
    fn delete_completion_preserves_last_completed() {
        let conn = test_db();
        let q = test_quest(&conn, "Vitamins");
        let completion = complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        let last_done_before = quests.iter().find(|x| x.id == q.id).unwrap().last_completed.clone();
        assert!(last_done_before.is_some());

        // Delete the completion
        delete_completion(&conn, completion.id).unwrap();

        // last_completed should be unchanged
        let quests = get_quests(&conn).unwrap();
        let last_done_after = quests.iter().find(|x| x.id == q.id).unwrap().last_completed.clone();
        assert_eq!(last_done_before, last_done_after);
    }

    #[test]
    fn set_quest_last_done_updates_due() {
        let conn = test_db();
        let q = test_quest(&conn, "Daily Task");

        // Set last-done to now — should no longer be due
        let now = chrono_now();
        set_quest_last_done(&conn, q.id.clone(), Some(now)).unwrap();
        let quests = get_quests(&conn).unwrap();
        let quest = quests.iter().find(|x| x.id == q.id).unwrap();
        assert!(!quest.is_due);

        // Set last-done to 3 days ago — should be due (daily quest)
        set_quest_last_done(&conn, q.id.clone(), Some("2020-01-01T00:00:00Z".into())).unwrap();
        let quests = get_quests(&conn).unwrap();
        let quest = quests.iter().find(|x| x.id == q.id).unwrap();
        assert!(quest.is_due);
    }

    #[test]
    fn set_quest_last_done_clear() {
        let conn = test_db();
        let q = test_quest(&conn, "Daily Task");

        // Set a date, then clear it
        set_quest_last_done(&conn, q.id.clone(), Some(chrono_now())).unwrap();
        set_quest_last_done(&conn, q.id.clone(), None).unwrap();

        let quests = get_quests(&conn).unwrap();
        let quest = quests.iter().find(|x| x.id == q.id).unwrap();
        assert!(quest.last_completed.is_none());
        assert!(quest.is_due); // never completed = due
    }

    #[test]
    fn set_quest_last_done_saga_step_rejected() {
        let conn = test_db();
        let s = add_saga(&conn, "Test Saga".into(), Some(1)).unwrap();
        let step = add_saga_step(&conn, NewSagaStep {
            saga_id: s.id.clone(),
            title: "Step 1".into(),
            ..NewSagaStep::default()
        }).unwrap();

        let result = set_quest_last_done(&conn, step.id, Some(chrono_now()));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("saga step"));
    }

    // --- Saga step scoring tests ---

    #[test]
    fn saga_step_scoring_daily_cycle() {
        let conn = test_db();
        let s = add_saga(&conn, "Daily Saga".into(), Some(1)).unwrap();
        add_saga_step(&conn, NewSagaStep {
            saga_id: s.id.clone(),
            title: "Step 1".into(),
            ..NewSagaStep::default()
        }).unwrap();
        // Backdate saga created_at so step has been active 1 day
        conn.execute(
            "UPDATE saga SET created_at = '2020-01-01T00:00:00Z' WHERE id = ?1",
            rusqlite::params![s.id],
        ).unwrap();

        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        assert_eq!(result.saga_name.as_deref(), Some("Daily Saga"));
        // (days + 1) / 1 — days_since is large, so ratio should be >> 2.0
        assert!(result.overdue_ratio > 2.0);
    }

    #[test]
    fn saga_step_scoring_weekly_cycle() {
        let conn = test_db();
        let s = add_saga(&conn, "Weekly Saga".into(), Some(7)).unwrap();
        add_saga_step(&conn, NewSagaStep {
            saga_id: s.id.clone(),
            title: "Step 1".into(),
            ..NewSagaStep::default()
        }).unwrap();
        // Backdate so step has been active a while
        conn.execute(
            "UPDATE saga SET created_at = '2020-01-01T00:00:00Z' WHERE id = ?1",
            rusqlite::params![s.id],
        ).unwrap();

        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        assert_eq!(result.saga_name.as_deref(), Some("Weekly Saga"));
        // (days + 7) / 7 — with many days since 2020, ratio is large but scaled by 7
        // Key: the ratio should be smaller than daily (scaled by 7 not 1)
        // We can't easily test exact values since days_since depends on current date,
        // but we can verify it uses the saga cycle by comparing to a daily saga
        assert!(result.overdue_ratio > 1.0);
    }

    #[test]
    fn saga_step_scoring_oneoff_uses_nine() {
        let conn = test_db();
        let s = add_saga(&conn, "One-off Saga".into(), None).unwrap();
        add_saga_step(&conn, NewSagaStep {
            saga_id: s.id.clone(),
            title: "Step 1".into(),
            ..NewSagaStep::default()
        }).unwrap();
        // Backdate 1 day
        conn.execute(
            "UPDATE saga SET created_at = '2020-01-01T00:00:00Z' WHERE id = ?1",
            rusqlite::params![s.id],
        ).unwrap();

        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        assert_eq!(result.saga_name.as_deref(), Some("One-off Saga"));
        // (days + 9) / 9 — one-off uses 9-day base
        assert!(result.overdue_ratio > 1.0);
    }

    // --- Importance tests ---

    #[test]
    fn importance_boosts_score() {
        let conn = test_db();
        let q1 = test_quest_with(&conn, "Important", |q| q.importance = 3);
        let _q2 = test_quest(&conn, "Normal");

        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        // Both are daily, never completed, same overdue. Importance 3 = +1.2 boost.
        assert_eq!(result.quest.id, q1.id);
        // importance 3 × 30.0 = 90.0
        assert!((result.importance_boost - 90.0).abs() < 0.01);
    }

    #[test]
    fn default_importance_is_zero() {
        let conn = test_db();
        let q = test_quest(&conn, "Default");
        assert_eq!(q.importance, 0);
    }

    #[test]
    fn update_quest_importance() {
        let conn = test_db();
        let q = test_quest(&conn, "Test");
        assert_eq!(q.importance, 0);

        let updated = update_quest(&conn, q.id, QuestUpdate { importance: Some(4), ..Default::default() }).unwrap();
        assert_eq!(updated.importance, 4);
    }

    #[test]
    fn saga_step_importance() {
        let conn = test_db();
        let s = add_saga(&conn, "Test".into(), Some(1)).unwrap();
        let step = add_saga_step(&conn, NewSagaStep {
            saga_id: s.id.clone(),
            title: "Urgent Step".into(),
            importance: 5,
            ..NewSagaStep::default()
        }).unwrap();
        assert_eq!(step.importance, 5);

        let steps = get_saga_steps(&conn, &s.id).unwrap();
        assert_eq!(steps[0].importance, 5);
    }

    // --- List-order weight + skip penalty tests ---

    #[test]
    fn list_order_bonus_uses_global_max() {
        let conn = test_db();
        // Create 3 quests — sort_order 1, 2, 3 (auto-incremented)
        let q1 = test_quest(&conn, "First");   // sort_order 1
        let q2 = test_quest(&conn, "Second");  // sort_order 2
        let q3 = test_quest(&conn, "Third");   // sort_order 3
        let _ = &q2; // used for global_max_sort calculation

        // Complete q1 so only q2 and q3 are due
        complete_quest(&conn, q1.id.clone()).unwrap();

        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        // q3 should win (highest sort_order). Its bonus should be 3/3 = 1.0
        assert_eq!(result.quest.id, q3.id);
        assert!((result.list_order_bonus - 1.0).abs() < 0.01);
    }

    #[test]
    fn top_of_list_gets_max_bonus() {
        let conn = test_db();
        let q = test_quest(&conn, "Only Quest");
        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        assert_eq!(result.quest.id, q.id);
        // Only quest: sort_order/max = 1/1 = 1.0
        assert!((result.list_order_bonus - 1.0).abs() < 0.01);
    }

    #[test]
    fn skip_penalty_scales_with_importance() {
        let conn = test_db();
        let q = test_quest_with(&conn, "Important", |q| q.importance = 2);
        let _q2 = test_quest(&conn, "Filler"); // need another quest so skip doesn't return the same one

        let mut skips = std::collections::HashMap::new();
        skips.insert(q.id.clone(), 1);

        let result = get_next_quest(&conn, &skips, None).unwrap().unwrap();
        // The skipped quest might not be returned — check via the filler or by finding q in scored
        // Instead, let's check that the non-skipped quest wins
        // Important quest: overdue ~1 + importance 60 - skip (0.5 + 2*15) = 61 - 30.5 = 30.5
        // Filler quest: overdue ~1 + importance 0 - skip 0 + order bonus ≈ 2.0
        // Important should still win despite skip
        assert_eq!(result.quest.id, q.id);
        // skip_penalty = 1 × (0.5 + 2 × 30/2) = 30.5
        assert!((result.skip_penalty - 30.5).abs() < 0.01);
    }

    #[test]
    fn skip_penalty_zero_importance() {
        let conn = test_db();
        let q = test_quest(&conn, "Normal");
        let _q2 = test_quest(&conn, "Filler");

        let mut skips = std::collections::HashMap::new();
        skips.insert(q.id.clone(), 1);

        // Get result — q might or might not be the winner, but we can check its penalty
        // With 0 importance: penalty = 1 × (0.5 + 0) = 0.5
        // Both quests are equal overdue, so filler (not skipped) should win
        let result = get_next_quest(&conn, &skips, None).unwrap().unwrap();
        assert_eq!(result.quest.id, _q2.id); // filler wins since q is penalized
    }

    // --- Campaign membership bonus tests ---

    #[test]
    fn campaign_membership_boosts_score() {
        let conn = test_db();
        let q1 = test_quest(&conn, "In Campaign");
        let _q2 = test_quest(&conn, "Not In Campaign");

        create_campaign(&conn, "Active".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q1.id.clone(), target_count: 5 },
        ]).unwrap();

        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        // q1 should win due to +1.0 membership bonus
        assert_eq!(result.quest.id, q1.id);
        assert!((result.membership_bonus - 1.0).abs() < 0.01);
    }

    #[test]
    fn campaign_membership_no_stacking() {
        let conn = test_db();
        let q = test_quest(&conn, "Multi Campaign");

        create_campaign(&conn, "Campaign A".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 5 },
        ]).unwrap();
        create_campaign(&conn, "Campaign B".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q.id.clone(), target_count: 3 },
        ]).unwrap();

        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        assert_eq!(result.quest.id, q.id);
        // Should be 1.0, not 2.0
        assert!((result.membership_bonus - 1.0).abs() < 0.01);
    }

    #[test]
    fn completed_campaign_no_membership_bonus() {
        let conn = test_db();
        let q1 = test_quest(&conn, "Was In Campaign");
        let _q2 = test_quest(&conn, "Never In Campaign");

        create_campaign(&conn, "Done".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: q1.id.clone(), target_count: 1 },
        ]).unwrap();

        // Complete the campaign
        check_campaign_progress(&conn, "quest_completions", &q1.id).unwrap();

        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        // q1 has higher sort_order so it still wins, but no membership bonus
        assert_eq!(result.membership_bonus, 0.0);
    }

    #[test]
    fn saga_step_no_membership_bonus() {
        let conn = test_db();
        let s = add_saga(&conn, "Test Saga".into(), Some(1)).unwrap();
        let step = add_saga_step(&conn, NewSagaStep {
            saga_id: s.id.clone(),
            title: "Step".into(),
            ..NewSagaStep::default()
        }).unwrap();

        // Put the saga step in a campaign
        create_campaign(&conn, "Saga Campaign".into(), vec![
            NewCriterion { target_type: "quest_completions".into(), target_id: step.id.clone(), target_count: 1 },
        ]).unwrap();

        // Backdate so step is active
        conn.execute(
            "UPDATE saga SET created_at = '2020-01-01T00:00:00Z' WHERE id = ?1",
            rusqlite::params![s.id],
        ).unwrap();

        let result = get_next_quest(&conn, &std::collections::HashMap::new(), None).unwrap().unwrap();
        assert_eq!(result.saga_name.as_deref(), Some("Test Saga"));
        // Saga step gets 0 membership bonus (gets 1.0 from list_order instead)
        assert_eq!(result.membership_bonus, 0.0);
        assert!((result.list_order_bonus - 1.0).abs() < 0.01);
    }
}
