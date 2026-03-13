use rusqlite::Connection;
use serde::Serialize;
use std::path::Path;
use uuid::Uuid;

#[derive(Serialize, Debug)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub cycle_days: Option<i32>,
    pub sort_order: i32,
    pub active: bool,
    pub created_at: String,
    pub last_completed: Option<String>,
    pub is_due: bool,
}

pub fn init_db(db_path: &Path) -> Connection {
    let conn = Connection::open(db_path).expect("Failed to open database");
    create_tables(&conn);
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
            cycle_days  INTEGER,
            sort_order  INTEGER NOT NULL,
            active      INTEGER NOT NULL DEFAULT 1,
            created_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS quest_completion (
            id            TEXT PRIMARY KEY,
            quest_id      TEXT NOT NULL REFERENCES quest(id),
            completed_at  TEXT NOT NULL
        );",
    )
    .expect("Failed to create tables");
}

pub fn get_quests(conn: &Connection) -> Result<Vec<Quest>, String> {
    let now = chrono_now();
    let mut stmt = conn
        .prepare(
            "SELECT q.id, q.title, q.cycle_days, q.sort_order, q.active, q.created_at,
                    (SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = q.id) as last_completed
             FROM quest q
             WHERE q.active = 1 OR (q.active = 0 AND q.cycle_days IS NULL)
             ORDER BY q.active DESC, q.sort_order DESC",
        )
        .map_err(|e| e.to_string())?;

    let quests = stmt
        .query_map([], |row| {
            let cycle_days: Option<i32> = row.get(2)?;
            let active = row.get::<_, i32>(4)? != 0;
            let last_completed: Option<String> = row.get(6)?;
            let is_due = compute_is_due(cycle_days, last_completed.as_deref(), &now);
            Ok(Quest {
                id: row.get(0)?,
                title: row.get(1)?,
                cycle_days,
                sort_order: row.get(3)?,
                active,
                created_at: row.get(5)?,
                last_completed,
                is_due,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(quests)
}

pub fn complete_quest(conn: &Connection, quest_id: String) -> Result<Quest, String> {
    // Verify quest exists
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

    let completion_id = Uuid::new_v4().to_string();
    let completed_at = chrono_now();

    conn.execute(
        "INSERT INTO quest_completion (id, quest_id, completed_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![completion_id, quest_id, completed_at],
    )
    .map_err(|e| e.to_string())?;

    // If one-off quest, deactivate it
    conn.execute(
        "UPDATE quest SET active = 0 WHERE id = ?1 AND cycle_days IS NULL",
        rusqlite::params![quest_id],
    )
    .map_err(|e| e.to_string())?;

    // Return the updated quest
    let now = chrono_now();
    conn.query_row(
        "SELECT q.id, q.title, q.cycle_days, q.sort_order, q.active, q.created_at,
                (SELECT MAX(completed_at) FROM quest_completion WHERE quest_id = q.id) as last_completed
         FROM quest q WHERE q.id = ?1",
        rusqlite::params![quest_id],
        |row| {
            let cycle_days: Option<i32> = row.get(2)?;
            let active = row.get::<_, i32>(4)? != 0;
            let last_completed: Option<String> = row.get(6)?;
            let is_due = compute_is_due(cycle_days, last_completed.as_deref(), &now);
            Ok(Quest {
                id: row.get(0)?,
                title: row.get(1)?,
                cycle_days,
                sort_order: row.get(3)?,
                active,
                created_at: row.get(5)?,
                last_completed,
                is_due,
            })
        },
    )
    .map_err(|e| e.to_string())
}

/// Determines if a quest is due/refreshed.
/// - One-off quests that are still active are always due.
/// - Recurring quests are due if never completed, or if cycle has elapsed.
/// - Completed one-offs are never due.
fn compute_is_due(cycle_days: Option<i32>, last_completed: Option<&str>, now: &str) -> bool {
    match (cycle_days, last_completed) {
        // Recurring, never completed → due
        (Some(_), None) => true,
        // Recurring, completed before → check if cycle elapsed
        (Some(cycle), Some(last)) => {
            if let (Some(last_days), Some(now_days)) = (date_to_days(last), date_to_days(now)) {
                now_days - last_days >= cycle as i64
            } else {
                true // if we can't parse dates, assume due
            }
        }
        // One-off, never completed → due
        (None, None) => true,
        // One-off, completed → not due (it's done)
        (None, Some(_)) => false,
    }
}

/// Extracts the day count from an ISO 8601 date string for simple day-level comparison.
fn date_to_days(iso: &str) -> Option<i64> {
    // Parse "YYYY-MM-DDT..." → extract year, month, day
    let parts: Vec<&str> = iso.split('T').next()?.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;

    // Approximate days since epoch (good enough for cycle comparison)
    Some(y * 365 + y / 4 - y / 100 + y / 400 + m * 30 + d)
}

pub fn add_quest(
    conn: &Connection,
    title: String,
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

    conn.execute(
        "INSERT INTO quest (id, title, cycle_days, sort_order, active, created_at)
         VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        rusqlite::params![id, title, cycle_days, sort_order, created_at],
    )
    .map_err(|e| e.to_string())?;

    Ok(Quest {
        id,
        title,
        cycle_days,
        sort_order,
        active: true,
        created_at,
        last_completed: None,
        is_due: true,
    })
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
        let quests = get_quests(&conn).unwrap();
        assert!(quests.is_empty());
    }

    #[test]
    fn add_and_retrieve_recurring_quest() {
        let conn = test_db();
        let quest = add_quest(&conn, "Take a shower".into(), Some(1)).unwrap();

        assert_eq!(quest.title, "Take a shower");
        assert_eq!(quest.cycle_days, Some(1));
        assert!(quest.active);

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests.len(), 1);
        assert_eq!(quests[0].title, "Take a shower");
        assert_eq!(quests[0].cycle_days, Some(1));
    }

    #[test]
    fn add_and_retrieve_one_off_quest() {
        let conn = test_db();
        let quest = add_quest(&conn, "File taxes".into(), None).unwrap();

        assert_eq!(quest.title, "File taxes");
        assert_eq!(quest.cycle_days, None);

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests.len(), 1);
        assert_eq!(quests[0].cycle_days, None);
    }

    #[test]
    fn quests_ordered_by_sort_order_descending() {
        let conn = test_db();
        add_quest(&conn, "First added".into(), Some(1)).unwrap();
        add_quest(&conn, "Second added".into(), Some(1)).unwrap();
        add_quest(&conn, "Third added".into(), Some(1)).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests.len(), 3);
        // Most recently added has highest sort_order, appears first
        assert_eq!(quests[0].title, "Third added");
        assert_eq!(quests[1].title, "Second added");
        assert_eq!(quests[2].title, "First added");
    }

    #[test]
    fn sort_order_auto_increments() {
        let conn = test_db();
        let q1 = add_quest(&conn, "First".into(), Some(1)).unwrap();
        let q2 = add_quest(&conn, "Second".into(), Some(1)).unwrap();
        let q3 = add_quest(&conn, "Third".into(), Some(1)).unwrap();

        assert_eq!(q1.sort_order, 1);
        assert_eq!(q2.sort_order, 2);
        assert_eq!(q3.sort_order, 3);
    }

    #[test]
    fn complete_recurring_quest_records_completion() {
        let conn = test_db();
        let quest = add_quest(&conn, "Take a shower".into(), Some(1)).unwrap();
        assert!(quest.is_due);
        assert!(quest.last_completed.is_none());

        let completed = complete_quest(&conn, quest.id.clone()).unwrap();
        assert!(completed.last_completed.is_some());
        assert!(completed.active); // recurring quests stay active
    }

    #[test]
    fn complete_one_off_quest_deactivates_it() {
        let conn = test_db();
        let quest = add_quest(&conn, "File taxes".into(), None).unwrap();
        assert!(quest.active);

        let completed = complete_quest(&conn, quest.id.clone()).unwrap();
        assert!(!completed.active);
        assert!(completed.last_completed.is_some());
    }

    #[test]
    fn completed_one_offs_still_appear_in_list() {
        let conn = test_db();
        let quest = add_quest(&conn, "File taxes".into(), None).unwrap();
        complete_quest(&conn, quest.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests.len(), 1);
        assert!(!quests[0].active);
        assert!(!quests[0].is_due);
    }

    #[test]
    fn recurring_quest_is_not_due_right_after_completion() {
        let conn = test_db();
        let quest = add_quest(&conn, "Take a shower".into(), Some(1)).unwrap();
        complete_quest(&conn, quest.id.clone()).unwrap();

        let quests = get_quests(&conn).unwrap();
        assert_eq!(quests.len(), 1);
        assert!(!quests[0].is_due); // just completed, cycle hasn't elapsed
    }

    #[test]
    fn multiple_completions_allowed() {
        let conn = test_db();
        let quest = add_quest(&conn, "Drink water".into(), Some(1)).unwrap();
        complete_quest(&conn, quest.id.clone()).unwrap();
        let second = complete_quest(&conn, quest.id.clone()).unwrap();

        assert!(second.last_completed.is_some());
        assert!(second.active);

        // Verify two completion records exist
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM quest_completion WHERE quest_id = ?1",
                rusqlite::params![quest.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn complete_nonexistent_quest_returns_error() {
        let conn = test_db();
        let result = complete_quest(&conn, "nonexistent-id".into());
        assert!(result.is_err());
    }

    #[test]
    fn is_due_logic() {
        // Never completed → due
        assert!(compute_is_due(Some(1), None, "2026-03-12T00:00:00Z"));
        // One-off never completed → due
        assert!(compute_is_due(None, None, "2026-03-12T00:00:00Z"));
        // One-off completed → not due
        assert!(!compute_is_due(None, Some("2026-03-12T00:00:00Z"), "2026-03-12T00:00:00Z"));
        // Completed today, 1-day cycle → not due yet
        assert!(!compute_is_due(Some(1), Some("2026-03-12T00:00:00Z"), "2026-03-12T23:59:00Z"));
        // Completed yesterday, 1-day cycle → due
        assert!(compute_is_due(Some(1), Some("2026-03-11T00:00:00Z"), "2026-03-12T00:00:00Z"));
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
