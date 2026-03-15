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

#[derive(Serialize, Debug)]
pub struct Completion {
    pub id: String,
    pub quest_id: Option<String>,
    pub quest_title: String,
    pub completed_at: String,
    pub xp_earned: i64,
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
    pub attribute_id: String,
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
            difficulty  TEXT NOT NULL DEFAULT 'easy'
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
            attribute_id  TEXT NOT NULL REFERENCES attribute(id),
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
            cta_interval_minutes  INTEGER NOT NULL DEFAULT 20
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
        ("Nature", "Connection", 7),
        ("Community", "Connection", 8),
        ("Sociality", "Connection", 9),
        ("Bureaucracy", "Responsibility", 10),
        ("Animal Handling", "Responsibility", 11),
        ("Logistics", "Responsibility", 12),
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

pub fn get_settings_db(conn: &Connection) -> Result<(bool, u64), String> {
    conn.query_row(
        "SELECT cta_enabled, cta_interval_minutes FROM settings WHERE id = 1",
        [],
        |row| {
            let enabled = row.get::<_, i32>(0)? != 0;
            let interval: u64 = row.get(1)?;
            Ok((enabled, interval))
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

pub fn get_quests(conn: &Connection) -> Result<Vec<Quest>, String> {
    let today = local_today_days();
    let mut stmt = conn
        .prepare(
            "SELECT q.id, q.title, q.quest_type, q.cycle_days, q.sort_order, q.active, q.created_at,
                    q.difficulty,
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
            let last_completed: Option<String> = row.get(8)?;
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

pub fn get_next_quest(conn: &Connection, skip_count: i32) -> Result<Option<Quest>, String> {
    let quests = get_quests(conn)?;
    let due: Vec<&Quest> = quests.iter().filter(|q| q.active && q.is_due).collect();

    if !due.is_empty() {
        let idx = (skip_count as usize) % due.len();
        return Ok(Some(due[idx].clone()));
    }

    // Fallback: active quest completed longest ago
    let mut active_with_completion: Vec<&Quest> = quests
        .iter()
        .filter(|q| q.active && q.last_completed.is_some())
        .collect();

    if active_with_completion.is_empty() {
        return Ok(None);
    }

    active_with_completion.sort_by(|a, b| {
        a.last_completed.as_deref().unwrap_or("").cmp(&b.last_completed.as_deref().unwrap_or(""))
    });

    Ok(Some(active_with_completion[0].clone()))
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
        "INSERT INTO quest (id, title, quest_type, cycle_days, sort_order, active, created_at, difficulty)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?7)",
        rusqlite::params![id, title, quest_type.as_str(), effective_cycle, sort_order, created_at, difficulty.as_str()],
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
        last_completed: None,
        is_due: true,
        skill_ids: Vec::new(),
        attribute_ids: Vec::new(),
    })
}

pub fn calculate_xp(difficulty: &Difficulty, quest_type: &QuestType, cycle_days: Option<i32>) -> i64 {
    let base: f64 = 10.0;

    let difficulty_mult: f64 = match difficulty {
        Difficulty::Trivial => 1.0,
        Difficulty::Easy => 2.0,
        Difficulty::Moderate => 4.0,
        Difficulty::Challenging => 7.0,
        Difficulty::Epic => 12.0,
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
        let (old_xp, attribute_id): (i64, String) = conn
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

        // Skill leveled up — award 70 XP bump to mapped attribute
        if level_after > level_before {
            conn.execute(
                "UPDATE attribute SET xp = xp + ?1 WHERE id = ?2",
                rusqlite::params![70, attribute_id],
            )
            .map_err(|e| e.to_string())?;
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
    let xp_earned = calculate_xp(&difficulty, &quest_type, cycle_days);

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

    Ok(Completion {
        id: completion_id,
        quest_id: Some(quest_id),
        quest_title,
        completed_at,
        xp_earned,
    })
}

pub fn update_quest(
    conn: &Connection,
    quest_id: String,
    title: Option<String>,
    quest_type: Option<QuestType>,
    cycle_days: Option<i32>,
    difficulty: Option<Difficulty>,
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
                q.difficulty,
                (SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = q.id) as last_completed
         FROM quest q WHERE q.id = ?1",
        rusqlite::params![quest_id],
        |row| {
            let quest_type_str: String = row.get(2)?;
            let quest_type = QuestType::from_str(&quest_type_str);
            let cycle_days: Option<i32> = row.get(3)?;
            let active = row.get::<_, i32>(5)? != 0;
            let difficulty_str: String = row.get(7)?;
            let last_completed: Option<String> = row.get(8)?;
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
        LevelScale::Attribute => (60, 100),
        LevelScale::Skill => (30, 50),
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
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        assert_eq!(q.quest_type, QuestType::Recurring);
        assert_eq!(q.cycle_days, Some(1));
        assert!(q.active);
        assert!(q.is_due);
    }

    #[test]
    fn add_recurring_quest_defaults_cycle_to_1() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, None, Difficulty::Easy).unwrap();
        assert_eq!(q.cycle_days, Some(1));
    }

    #[test]
    fn add_one_off_quest() {
        let conn = test_db();
        let q = add_quest(&conn, "File taxes".into(), QuestType::OneOff, None, Difficulty::Easy).unwrap();
        assert_eq!(q.quest_type, QuestType::OneOff);
        assert_eq!(q.cycle_days, None);
        assert!(q.is_due);
    }

    #[test]
    fn add_one_off_ignores_cycle_days() {
        let conn = test_db();
        let q = add_quest(&conn, "Taxes".into(), QuestType::OneOff, Some(5), Difficulty::Easy).unwrap();
        assert_eq!(q.cycle_days, None); // ignored for one-off
    }

    #[test]
    fn quests_ordered_by_sort_order_descending() {
        let conn = test_db();
        add_quest(&conn, "First".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        add_quest(&conn, "Second".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        add_quest(&conn, "Third".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests[0].title, "Third");
        assert_eq!(quests[1].title, "Second");
        assert_eq!(quests[2].title, "First");
    }

    #[test]
    fn sort_order_auto_increments() {
        let conn = test_db();
        let q1 = add_quest(&conn, "A".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let q2 = add_quest(&conn, "B".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        assert_eq!(q1.sort_order, 1);
        assert_eq!(q2.sort_order, 2);
    }

    // --- Completion tests ---

    #[test]
    fn complete_quest_snapshots_title() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let c = complete_quest(&conn, q.id.clone()).unwrap();
        assert_eq!(c.quest_title, "Shower");
        assert_eq!(c.quest_id, Some(q.id));
    }

    #[test]
    fn complete_recurring_stays_active() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert!(quests[0].active);
        assert!(!quests[0].is_due);
    }

    #[test]
    fn complete_one_off_deactivates() {
        let conn = test_db();
        let q = add_quest(&conn, "Taxes".into(), QuestType::OneOff, None, Difficulty::Easy).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert!(!quests[0].active);
        assert!(!quests[0].is_due);
    }

    #[test]
    fn multiple_completions() {
        let conn = test_db();
        let q = add_quest(&conn, "Water".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Delete me".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Old".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let u = update_quest(&conn, q.id, Some("New".into()), None, None, None).unwrap();
        assert_eq!(u.title, "New");
        assert_eq!(u.cycle_days, Some(1));
    }

    #[test]
    fn update_quest_cycle() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let u = update_quest(&conn, q.id, None, None, Some(3), None).unwrap();
        assert_eq!(u.cycle_days, Some(3));
    }

    #[test]
    fn update_quest_type_to_one_off() {
        let conn = test_db();
        let q = add_quest(&conn, "Maybe once".into(), QuestType::Recurring, Some(7), Difficulty::Easy).unwrap();
        let u = update_quest(&conn, q.id, None, Some(QuestType::OneOff), None, None).unwrap();
        assert_eq!(u.quest_type, QuestType::OneOff);
        assert_eq!(u.cycle_days, None); // cleared
    }

    #[test]
    fn update_quest_type_to_recurring() {
        let conn = test_db();
        let q = add_quest(&conn, "Now recurring".into(), QuestType::OneOff, None, Difficulty::Easy).unwrap();
        let u = update_quest(&conn, q.id, None, Some(QuestType::Recurring), Some(3), None).unwrap();
        assert_eq!(u.quest_type, QuestType::Recurring);
        assert_eq!(u.cycle_days, Some(3));
    }

    #[test]
    fn update_quest_type_to_recurring_defaults_cycle() {
        let conn = test_db();
        let q = add_quest(&conn, "Now recurring".into(), QuestType::OneOff, None, Difficulty::Easy).unwrap();
        let u = update_quest(&conn, q.id, None, Some(QuestType::Recurring), None, None).unwrap();
        assert_eq!(u.quest_type, QuestType::Recurring);
        assert_eq!(u.cycle_days, Some(1)); // default
    }

    #[test]
    fn update_nonexistent_errors() {
        let conn = test_db();
        assert!(update_quest(&conn, "nope".into(), Some("x".into()), None, None, None).is_err());
    }

    // --- Reorder ---

    #[test]
    fn reorder_quests_swaps_order() {
        let conn = test_db();
        let a = add_quest(&conn, "A".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let b = add_quest(&conn, "B".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Real".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        assert_eq!(attrs.len(), 5);
        let names: Vec<&str> = attrs.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Health", "Pluck", "Knowledge", "Connection", "Responsibility"]);
    }

    #[test]
    fn seed_data_creates_skills() {
        let conn = test_db();
        let skills = get_skills(&conn).unwrap();
        assert_eq!(skills.len(), 12);
        assert_eq!(skills[0].name, "Cooking");
        assert_eq!(skills[11].name, "Logistics");
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
        let info = level_from_xp(0, &LevelScale::Attribute);
        assert_eq!(info.level, 1);
        assert_eq!(info.xp_for_current_level, 60);

        let info = level_from_xp(60, &LevelScale::Attribute);
        assert_eq!(info.level, 2);
        assert_eq!(info.xp_for_current_level, 100);

        let info = level_from_xp(160, &LevelScale::Attribute);
        assert_eq!(info.level, 3);
        assert_eq!(info.xp_for_current_level, 160);
    }

    #[test]
    fn level_from_xp_skill_scale() {
        let info = level_from_xp(0, &LevelScale::Skill);
        assert_eq!(info.level, 1);
        assert_eq!(info.xp_for_current_level, 30);

        let info = level_from_xp(30, &LevelScale::Skill);
        assert_eq!(info.level, 2);
        assert_eq!(info.xp_for_current_level, 50);

        let info = level_from_xp(80, &LevelScale::Skill);
        assert_eq!(info.level, 3);
        assert_eq!(info.xp_for_current_level, 80);
    }

    // --- Difficulty ---

    #[test]
    fn add_quest_with_difficulty() {
        let conn = test_db();
        let q = add_quest(&conn, "Hard task".into(), QuestType::OneOff, None, Difficulty::Epic).unwrap();
        assert_eq!(q.difficulty, Difficulty::Epic);

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests[0].difficulty, Difficulty::Epic);
    }

    #[test]
    fn quest_defaults_to_easy() {
        let conn = test_db();
        let q = add_quest(&conn, "Simple".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        assert_eq!(q.difficulty, Difficulty::Easy);
    }

    #[test]
    fn update_quest_difficulty() {
        let conn = test_db();
        let q = add_quest(&conn, "Task".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let u = update_quest(&conn, q.id, None, None, None, Some(Difficulty::Challenging)).unwrap();
        assert_eq!(u.difficulty, Difficulty::Challenging);
    }

    // --- Quest links ---

    #[test]
    fn set_and_get_quest_links() {
        let conn = test_db();
        let q = add_quest(&conn, "Linked".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Replace".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "With links".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Delete links".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let skills = get_skills(&conn).unwrap();

        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();
        delete_quest(&conn, q.id.clone()).unwrap();

        // Verify link rows are gone (create another quest to reuse the skill)
        let q2 = add_quest(&conn, "New".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Bad link".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let result = set_quest_links(&conn, q.id, vec!["fake-skill".into()], vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn set_quest_links_invalid_attribute_errors() {
        let conn = test_db();
        let q = add_quest(&conn, "Bad link".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let result = set_quest_links(&conn, q.id, vec![], vec!["fake-attr".into()]);
        assert!(result.is_err());
    }

    // --- XP Engine ---

    #[test]
    fn calculate_xp_daily_difficulties() {
        // Daily recurring (cycle_days=1, sqrt(1)=1.0)
        assert_eq!(calculate_xp(&Difficulty::Trivial, &QuestType::Recurring, Some(1)), 10);
        assert_eq!(calculate_xp(&Difficulty::Easy, &QuestType::Recurring, Some(1)), 20);
        assert_eq!(calculate_xp(&Difficulty::Moderate, &QuestType::Recurring, Some(1)), 40);
        assert_eq!(calculate_xp(&Difficulty::Challenging, &QuestType::Recurring, Some(1)), 70);
        assert_eq!(calculate_xp(&Difficulty::Epic, &QuestType::Recurring, Some(1)), 120);
    }

    #[test]
    fn calculate_xp_one_off() {
        // One-off: cycle_mult=3.0
        assert_eq!(calculate_xp(&Difficulty::Easy, &QuestType::OneOff, None), 60);
        assert_eq!(calculate_xp(&Difficulty::Epic, &QuestType::OneOff, None), 360);
    }

    #[test]
    fn calculate_xp_multi_day_recurring() {
        // 7-day cycle: sqrt(7) ≈ 2.6458
        // Easy: 10 * 2 * 2.6458 = 52.915 → 53
        assert_eq!(calculate_xp(&Difficulty::Easy, &QuestType::Recurring, Some(7)), 53);
        // 30-day cycle: sqrt(30) ≈ 5.4772
        // Trivial: 10 * 1 * 5.4772 = 54.772 → 55
        assert_eq!(calculate_xp(&Difficulty::Trivial, &QuestType::Recurring, Some(30)), 55);
    }

    #[test]
    fn award_xp_character_only_no_links() {
        let conn = test_db();
        let q = add_quest(&conn, "Solo".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Linked".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Grind".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let skills = get_skills(&conn).unwrap();
        // Cooking (skill[0]) maps to Health (attr[0])
        // Skill level 2 at 30 XP. Award 30 to trigger level-up.
        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();

        award_xp(&conn, &q.id, 30).unwrap();

        let updated_skills = get_skills(&conn).unwrap();
        assert_eq!(updated_skills[0].xp, 30);
        assert_eq!(updated_skills[0].level, 2);

        // Health should have received 70 XP bump
        let updated_attrs = get_attributes(&conn).unwrap();
        assert_eq!(updated_attrs[0].xp, 70);
    }

    #[test]
    fn complete_quest_stores_xp_earned() {
        let conn = test_db();
        let q = add_quest(&conn, "XP test".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let c = complete_quest(&conn, q.id).unwrap();

        // Easy daily = 20 XP
        assert_eq!(c.xp_earned, 20);

        let completions = get_completions(&conn).unwrap();
        assert_eq!(completions[0].xp_earned, 20);
    }

    #[test]
    fn complete_quest_awards_character_xp() {
        let conn = test_db();
        let q = add_quest(&conn, "XP flow".into(), QuestType::Recurring, Some(1), Difficulty::Moderate).unwrap();
        complete_quest(&conn, q.id).unwrap();

        // Moderate daily = 40 XP
        let c = get_character(&conn).unwrap();
        assert_eq!(c.xp, 40);
    }

    #[test]
    fn delete_completion_does_not_reduce_xp() {
        let conn = test_db();
        let q = add_quest(&conn, "Permanent".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let c = complete_quest(&conn, q.id).unwrap();

        let char_before = get_character(&conn).unwrap();
        assert_eq!(char_before.xp, 20);

        delete_completion(&conn, c.id).unwrap();

        let char_after = get_character(&conn).unwrap();
        assert_eq!(char_after.xp, 20); // unchanged
    }

    // --- Reset functions ---

    #[test]
    fn reset_character_zeroes_all_xp() {
        let conn = test_db();
        let q = add_quest(&conn, "XP".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
        let q = add_quest(&conn, "Gone".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        let skills = get_skills(&conn).unwrap();
        set_quest_links(&conn, q.id.clone(), vec![skills[0].id.clone()], vec![]).unwrap();

        reset_quests(&conn).unwrap();

        assert!(get_quests(&conn).unwrap().is_empty());
    }

    #[test]
    fn reset_completions_deletes_all() {
        let conn = test_db();
        let q = add_quest(&conn, "Done".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
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
    fn get_next_quest_returns_first_due() {
        let conn = test_db();
        add_quest(&conn, "First".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        add_quest(&conn, "Second".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();

        let next = get_next_quest(&conn, 0).unwrap().unwrap();
        // sort_order DESC means higher sort_order first — Second was added last with higher sort_order
        assert_eq!(next.title, "Second");
    }

    #[test]
    fn get_next_quest_skip_cycles() {
        let conn = test_db();
        add_quest(&conn, "First".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();
        add_quest(&conn, "Second".into(), QuestType::Recurring, Some(1), Difficulty::Easy).unwrap();

        let q0 = get_next_quest(&conn, 0).unwrap().unwrap();
        let q1 = get_next_quest(&conn, 1).unwrap().unwrap();
        assert_ne!(q0.id, q1.id);
        // Wraps around
        let q2 = get_next_quest(&conn, 2).unwrap().unwrap();
        assert_eq!(q0.id, q2.id);
    }

    #[test]
    fn get_next_quest_empty_db() {
        let conn = test_db();
        assert!(get_next_quest(&conn, 0).unwrap().is_none());
    }

    #[test]
    fn get_next_quest_none_due_falls_back() {
        let conn = test_db();
        // Add a quest with a long cycle so it won't be due after completion
        let q = add_quest(&conn, "Long cycle".into(), QuestType::Recurring, Some(999), Difficulty::Easy).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        // Not due, but should fall back to longest-ago-completed
        let next = get_next_quest(&conn, 0).unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().id, q.id);
    }
}
