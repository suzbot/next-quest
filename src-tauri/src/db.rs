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

#[derive(Serialize, Debug)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub quest_type: QuestType,
    pub cycle_days: Option<i32>,
    pub sort_order: i32,
    pub active: bool,
    pub created_at: String,
    pub last_completed: Option<String>,
    pub is_due: bool,
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
}

pub fn init_db(db_path: &Path) -> Connection {
    let conn = Connection::open(db_path).expect("Failed to open database");
    create_tables(&conn);
    migrate(&conn);
    conn
}

pub fn init_db_memory() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory database");
    create_tables(&conn);
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
            created_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS quest_completion (
            id            TEXT PRIMARY KEY,
            quest_id      TEXT,
            quest_title   TEXT NOT NULL DEFAULT '',
            completed_at  TEXT NOT NULL
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
}

pub fn get_quests(conn: &Connection) -> Result<Vec<Quest>, String> {
    let now = chrono_now();
    let mut stmt = conn
        .prepare(
            "SELECT q.id, q.title, q.quest_type, q.cycle_days, q.sort_order, q.active, q.created_at,
                    (SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = q.id) as last_completed
             FROM quest q
             WHERE q.active = 1 OR (q.active = 0 AND q.quest_type = 'one_off')
             ORDER BY q.active DESC, q.sort_order DESC",
        )
        .map_err(|e| e.to_string())?;

    let quests = stmt
        .query_map([], |row| {
            let quest_type_str: String = row.get(2)?;
            let quest_type = QuestType::from_str(&quest_type_str);
            let cycle_days: Option<i32> = row.get(3)?;
            let active = row.get::<_, i32>(5)? != 0;
            let last_completed: Option<String> = row.get(7)?;
            let is_due = compute_is_due(&quest_type, active, last_completed.as_deref(), cycle_days, &now);
            Ok(Quest {
                id: row.get(0)?,
                title: row.get(1)?,
                quest_type,
                cycle_days,
                sort_order: row.get(4)?,
                active,
                created_at: row.get(6)?,
                last_completed,
                is_due,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(quests)
}

pub fn get_completions(conn: &Connection) -> Result<Vec<Completion>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, quest_id, quest_title, completed_at
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
        "INSERT INTO quest (id, title, quest_type, cycle_days, sort_order, active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
        rusqlite::params![id, title, quest_type.as_str(), effective_cycle, sort_order, created_at],
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
        last_completed: None,
        is_due: true,
    })
}

pub fn complete_quest(conn: &Connection, quest_id: String) -> Result<Completion, String> {
    let quest_title: String = conn
        .query_row(
            "SELECT title FROM quest WHERE id = ?1",
            rusqlite::params![quest_id],
            |row| row.get(0),
        )
        .map_err(|_| format!("Quest not found: {}", quest_id))?;

    let completion_id = Uuid::new_v4().to_string();
    let completed_at = chrono_now();

    conn.execute(
        "INSERT INTO quest_completion (id, quest_id, quest_title, completed_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![completion_id, quest_id, quest_title, completed_at],
    )
    .map_err(|e| e.to_string())?;

    // If one-off quest, deactivate it
    conn.execute(
        "UPDATE quest SET active = 0 WHERE id = ?1 AND quest_type = 'one_off'",
        rusqlite::params![quest_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(Completion {
        id: completion_id,
        quest_id: Some(quest_id),
        quest_title,
        completed_at,
    })
}

pub fn update_quest(
    conn: &Connection,
    quest_id: String,
    title: Option<String>,
    quest_type: Option<QuestType>,
    cycle_days: Option<i32>,
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

    let now = chrono_now();
    query_single_quest(conn, &quest_id, &now)
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

fn query_single_quest(conn: &Connection, quest_id: &str, now: &str) -> Result<Quest, String> {
    conn.query_row(
        "SELECT q.id, q.title, q.quest_type, q.cycle_days, q.sort_order, q.active, q.created_at,
                (SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = q.id) as last_completed
         FROM quest q WHERE q.id = ?1",
        rusqlite::params![quest_id],
        |row| {
            let quest_type_str: String = row.get(2)?;
            let quest_type = QuestType::from_str(&quest_type_str);
            let cycle_days: Option<i32> = row.get(3)?;
            let active = row.get::<_, i32>(5)? != 0;
            let last_completed: Option<String> = row.get(7)?;
            let is_due = compute_is_due(&quest_type, active, last_completed.as_deref(), cycle_days, now);
            Ok(Quest {
                id: row.get(0)?,
                title: row.get(1)?,
                quest_type,
                cycle_days,
                sort_order: row.get(4)?,
                active,
                created_at: row.get(6)?,
                last_completed,
                is_due,
            })
        },
    )
    .map_err(|e| e.to_string())
}

fn compute_is_due(quest_type: &QuestType, active: bool, last_completed: Option<&str>, cycle_days: Option<i32>, now: &str) -> bool {
    if !active {
        return false;
    }
    match quest_type {
        QuestType::OneOff => last_completed.is_none(),
        QuestType::Recurring => {
            match (cycle_days, last_completed) {
                (_, None) => true,
                (Some(cycle), Some(last)) => {
                    if let (Some(last_days), Some(now_days)) = (date_to_days(last), date_to_days(now)) {
                        now_days - last_days >= cycle as i64
                    } else {
                        true
                    }
                }
                (None, Some(_)) => false,
            }
        }
    }
}

fn date_to_days(iso: &str) -> Option<i64> {
    let parts: Vec<&str> = iso.split('T').next()?.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;
    Some(y * 365 + y / 4 - y / 100 + y / 400 + m * 30 + d)
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
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1)).unwrap();
        assert_eq!(q.quest_type, QuestType::Recurring);
        assert_eq!(q.cycle_days, Some(1));
        assert!(q.active);
        assert!(q.is_due);
    }

    #[test]
    fn add_recurring_quest_defaults_cycle_to_1() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, None).unwrap();
        assert_eq!(q.cycle_days, Some(1));
    }

    #[test]
    fn add_one_off_quest() {
        let conn = test_db();
        let q = add_quest(&conn, "File taxes".into(), QuestType::OneOff, None).unwrap();
        assert_eq!(q.quest_type, QuestType::OneOff);
        assert_eq!(q.cycle_days, None);
        assert!(q.is_due);
    }

    #[test]
    fn add_one_off_ignores_cycle_days() {
        let conn = test_db();
        let q = add_quest(&conn, "Taxes".into(), QuestType::OneOff, Some(5)).unwrap();
        assert_eq!(q.cycle_days, None); // ignored for one-off
    }

    #[test]
    fn quests_ordered_by_sort_order_descending() {
        let conn = test_db();
        add_quest(&conn, "First".into(), QuestType::Recurring, Some(1)).unwrap();
        add_quest(&conn, "Second".into(), QuestType::Recurring, Some(1)).unwrap();
        add_quest(&conn, "Third".into(), QuestType::Recurring, Some(1)).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests[0].title, "Third");
        assert_eq!(quests[1].title, "Second");
        assert_eq!(quests[2].title, "First");
    }

    #[test]
    fn sort_order_auto_increments() {
        let conn = test_db();
        let q1 = add_quest(&conn, "A".into(), QuestType::Recurring, Some(1)).unwrap();
        let q2 = add_quest(&conn, "B".into(), QuestType::Recurring, Some(1)).unwrap();
        assert_eq!(q1.sort_order, 1);
        assert_eq!(q2.sort_order, 2);
    }

    // --- Completion tests ---

    #[test]
    fn complete_quest_snapshots_title() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1)).unwrap();
        let c = complete_quest(&conn, q.id.clone()).unwrap();
        assert_eq!(c.quest_title, "Shower");
        assert_eq!(c.quest_id, Some(q.id));
    }

    #[test]
    fn complete_recurring_stays_active() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1)).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert!(quests[0].active);
        assert!(!quests[0].is_due);
    }

    #[test]
    fn complete_one_off_deactivates() {
        let conn = test_db();
        let q = add_quest(&conn, "Taxes".into(), QuestType::OneOff, None).unwrap();
        complete_quest(&conn, q.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert!(!quests[0].active);
        assert!(!quests[0].is_due);
    }

    #[test]
    fn multiple_completions() {
        let conn = test_db();
        let q = add_quest(&conn, "Water".into(), QuestType::Recurring, Some(1)).unwrap();
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
        let q = add_quest(&conn, "Delete me".into(), QuestType::Recurring, Some(1)).unwrap();
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
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1)).unwrap();
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
        let q = add_quest(&conn, "Old".into(), QuestType::Recurring, Some(1)).unwrap();
        let u = update_quest(&conn, q.id, Some("New".into()), None, None).unwrap();
        assert_eq!(u.title, "New");
        assert_eq!(u.cycle_days, Some(1));
    }

    #[test]
    fn update_quest_cycle() {
        let conn = test_db();
        let q = add_quest(&conn, "Shower".into(), QuestType::Recurring, Some(1)).unwrap();
        let u = update_quest(&conn, q.id, None, None, Some(3)).unwrap();
        assert_eq!(u.cycle_days, Some(3));
    }

    #[test]
    fn update_quest_type_to_one_off() {
        let conn = test_db();
        let q = add_quest(&conn, "Maybe once".into(), QuestType::Recurring, Some(7)).unwrap();
        let u = update_quest(&conn, q.id, None, Some(QuestType::OneOff), None).unwrap();
        assert_eq!(u.quest_type, QuestType::OneOff);
        assert_eq!(u.cycle_days, None); // cleared
    }

    #[test]
    fn update_quest_type_to_recurring() {
        let conn = test_db();
        let q = add_quest(&conn, "Now recurring".into(), QuestType::OneOff, None).unwrap();
        let u = update_quest(&conn, q.id, None, Some(QuestType::Recurring), Some(3)).unwrap();
        assert_eq!(u.quest_type, QuestType::Recurring);
        assert_eq!(u.cycle_days, Some(3));
    }

    #[test]
    fn update_quest_type_to_recurring_defaults_cycle() {
        let conn = test_db();
        let q = add_quest(&conn, "Now recurring".into(), QuestType::OneOff, None).unwrap();
        let u = update_quest(&conn, q.id, None, Some(QuestType::Recurring), None).unwrap();
        assert_eq!(u.quest_type, QuestType::Recurring);
        assert_eq!(u.cycle_days, Some(1)); // default
    }

    #[test]
    fn update_nonexistent_errors() {
        let conn = test_db();
        assert!(update_quest(&conn, "nope".into(), Some("x".into()), None, None).is_err());
    }

    // --- Reorder ---

    #[test]
    fn reorder_quests_swaps_order() {
        let conn = test_db();
        let a = add_quest(&conn, "A".into(), QuestType::Recurring, Some(1)).unwrap();
        let b = add_quest(&conn, "B".into(), QuestType::Recurring, Some(1)).unwrap();
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
        let q = add_quest(&conn, "Real".into(), QuestType::Recurring, Some(1)).unwrap();
        let result = reorder_quests(&conn, vec![
            QuestOrder { id: q.id, sort_order: 5 },
            QuestOrder { id: "nonexistent".into(), sort_order: 3 },
        ]);
        assert!(result.is_err());
    }

    // --- is_due logic ---

    #[test]
    fn is_due_logic() {
        // Recurring, never completed
        assert!(compute_is_due(&QuestType::Recurring, true, None, Some(1), "2026-03-12T00:00:00Z"));
        // Recurring, just completed
        assert!(!compute_is_due(&QuestType::Recurring, true, Some("2026-03-12T00:00:00Z"), Some(1), "2026-03-12T23:59:00Z"));
        // Recurring, cycle elapsed
        assert!(compute_is_due(&QuestType::Recurring, true, Some("2026-03-11T00:00:00Z"), Some(1), "2026-03-12T00:00:00Z"));
        // One-off, never completed
        assert!(compute_is_due(&QuestType::OneOff, true, None, None, "2026-03-12T00:00:00Z"));
        // One-off, completed
        assert!(!compute_is_due(&QuestType::OneOff, true, Some("2026-03-12T00:00:00Z"), None, "2026-03-12T00:00:00Z"));
        // Inactive = never due
        assert!(!compute_is_due(&QuestType::Recurring, false, None, Some(1), "2026-03-12T00:00:00Z"));
        assert!(!compute_is_due(&QuestType::OneOff, false, None, None, "2026-03-12T00:00:00Z"));
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
}
