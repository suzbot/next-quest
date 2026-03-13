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
    let mut stmt = conn
        .prepare(
            "SELECT id, title, cycle_days, sort_order, active, created_at
             FROM quest
             WHERE active = 1
             ORDER BY sort_order DESC",
        )
        .map_err(|e| e.to_string())?;

    let quests = stmt
        .query_map([], |row| {
            Ok(Quest {
                id: row.get(0)?,
                title: row.get(1)?,
                cycle_days: row.get(2)?,
                sort_order: row.get(3)?,
                active: row.get::<_, i32>(4).map(|v| v != 0)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(quests)
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
