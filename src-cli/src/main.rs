use clap::{Parser, Subcommand};
use nq_core::db;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "nq", about = "Next Quest CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List quests as JSON (same view as the GUI quest list)
    ListQuests {
        /// Only active quests
        #[arg(long)]
        active: bool,
        /// Filter by difficulty (trivial, easy, moderate, challenging, epic)
        #[arg(long)]
        difficulty: Option<String>,
        /// Only quests currently due
        #[arg(long)]
        due: bool,
    },
    /// List sagas as JSON with progress info
    ListSagas,
    /// List tags as JSON
    ListTags,
    /// List skills as JSON
    ListSkills,
    /// List attributes as JSON
    ListAttributes,
    /// Create a new quest
    AddQuest {
        /// Quest title
        #[arg(long)]
        title: String,
        /// Difficulty: trivial, easy, moderate, challenging, epic
        #[arg(long)]
        difficulty: String,
        /// Quest type: recurring, one_off
        #[arg(long = "type")]
        quest_type: String,
        /// Importance 0-5
        #[arg(long, default_value = "0")]
        importance: i32,
        /// Cycle days (required for recurring, ignored for one_off)
        #[arg(long)]
        cycle_days: Option<i32>,
        /// Time of day: morning,afternoon,evening,night,anytime
        #[arg(long)]
        time_of_day: Option<String>,
        /// Days of week: mon,tue,wed,thu,fri,sat,sun,everyday
        #[arg(long)]
        days_of_week: Option<String>,
        /// Comma-separated tag names (auto-created if new)
        #[arg(long)]
        tags: Option<String>,
        /// Comma-separated skill names (must exist)
        #[arg(long)]
        skills: Option<String>,
        /// Comma-separated attribute names (must exist)
        #[arg(long)]
        attributes: Option<String>,
    },
}

// --- CLI output types (consistent snake_case) ---

#[derive(Serialize)]
struct QuestOutput {
    id: String,
    title: String,
    item_type: String,
    quest_type: String,
    difficulty: String,
    is_due: bool,
    active: bool,
    cycle_days: Option<i32>,
    importance: i32,
    sort_order: i32,
    created_at: String,
    time_of_day: Vec<String>,
    days_of_week: Vec<String>,
    last_completed: Option<String>,
    dismissed_today: bool,
    skills: Vec<String>,
    attributes: Vec<String>,
    tags: Vec<String>,
    saga_id: Option<String>,
    saga_name: Option<String>,
}

#[derive(Serialize)]
struct SagaOutput {
    id: String,
    name: String,
    cycle_days: Option<i32>,
    active: bool,
    is_due: bool,
    total_steps: usize,
    completed_steps: usize,
    created_at: String,
    last_run_completed_at: Option<String>,
}

#[derive(Serialize)]
struct TagOutput {
    id: String,
    name: String,
}

#[derive(Serialize)]
struct SkillOutput {
    id: String,
    name: String,
    attribute: Option<String>,
    xp: i64,
    level: i32,
    xp_for_current_level: i64,
    xp_into_current_level: i64,
}

#[derive(Serialize)]
struct AttributeOutput {
    id: String,
    name: String,
    xp: i64,
    level: i32,
    xp_for_current_level: i64,
    xp_into_current_level: i64,
}

// --- Bitmask-to-name helpers ---

fn time_of_day_names(mask: i32) -> Vec<String> {
    if mask == 0 || mask == 15 {
        return vec!["anytime".to_string()];
    }
    let mut names = Vec::new();
    if mask & 1 != 0 { names.push("morning".to_string()); }
    if mask & 2 != 0 { names.push("afternoon".to_string()); }
    if mask & 4 != 0 { names.push("evening".to_string()); }
    if mask & 8 != 0 { names.push("night".to_string()); }
    names
}

fn days_of_week_names(mask: i32) -> Vec<String> {
    if mask == 0 || mask == 127 {
        return vec!["everyday".to_string()];
    }
    let mut names = Vec::new();
    if mask & 1 != 0 { names.push("mon".to_string()); }
    if mask & 2 != 0 { names.push("tue".to_string()); }
    if mask & 4 != 0 { names.push("wed".to_string()); }
    if mask & 8 != 0 { names.push("thu".to_string()); }
    if mask & 16 != 0 { names.push("fri".to_string()); }
    if mask & 32 != 0 { names.push("sat".to_string()); }
    if mask & 64 != 0 { names.push("sun".to_string()); }
    names
}

// --- Name resolution helpers ---

fn build_id_name_map<T, F>(items: &[T], get_id: F) -> HashMap<String, String>
where
    F: Fn(&T) -> (String, String),
{
    items.iter().map(|item| get_id(item)).collect()
}

fn resolve_names(ids: &[String], map: &HashMap<String, String>) -> Vec<String> {
    ids.iter()
        .filter_map(|id| map.get(id).cloned())
        .collect()
}

fn quest_to_output(
    quest: &db::Quest,
    item_type: &str,
    dismissed_today: bool,
    saga_id: Option<&str>,
    saga_name: Option<&str>,
    skill_map: &HashMap<String, String>,
    attr_map: &HashMap<String, String>,
    tag_map: &HashMap<String, String>,
) -> QuestOutput {
    QuestOutput {
        id: quest.id.clone(),
        title: quest.title.clone(),
        item_type: item_type.to_string(),
        quest_type: quest.quest_type.as_str().to_string(),
        difficulty: quest.difficulty.as_str().to_string(),
        is_due: quest.is_due,
        active: quest.active,
        cycle_days: quest.cycle_days,
        importance: quest.importance,
        sort_order: quest.sort_order,
        created_at: quest.created_at.clone(),
        time_of_day: time_of_day_names(quest.time_of_day),
        days_of_week: days_of_week_names(quest.days_of_week),
        last_completed: quest.last_completed.clone(),
        dismissed_today,
        skills: resolve_names(&quest.skill_ids, skill_map),
        attributes: resolve_names(&quest.attribute_ids, attr_map),
        tags: resolve_names(&quest.tag_ids, tag_map),
        saga_id: saga_id.map(|s| s.to_string()),
        saga_name: saga_name.map(|s| s.to_string()),
    }
}

// --- Main ---

fn main() {
    let cli = Cli::parse();

    let result = run(cli.command);

    match result {
        Ok(json) => {
            println!("{}", json);
        }
        Err(e) => {
            let err = serde_json::json!({"ok": false, "error": e});
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn run(command: Commands) -> Result<String, String> {
    let db_path = nq_core::db_path();
    let conn = db::init_db(&db_path);

    match command {
        Commands::ListQuests { active, difficulty, due } => {
            list_quests(&conn, active, difficulty, due)
        }
        Commands::ListSagas => list_sagas(&conn),
        Commands::ListTags => list_tags(&conn),
        Commands::ListSkills => list_skills(&conn),
        Commands::ListAttributes => list_attributes(&conn),
        Commands::AddQuest {
            title, difficulty, quest_type, importance,
            cycle_days, time_of_day, days_of_week,
            tags, skills, attributes,
        } => {
            cmd_add_quest(
                &conn, title, difficulty, quest_type, importance,
                cycle_days, time_of_day, days_of_week,
                tags, skills, attributes,
            )
        }
    }
}

fn list_quests(
    conn: &nq_core::rusqlite::Connection,
    filter_active: bool,
    filter_difficulty: Option<String>,
    filter_due: bool,
) -> Result<String, String> {
    let items = db::get_quest_list(conn)?;
    let skills = db::get_skills(conn)?;
    let attributes = db::get_attributes(conn)?;
    let tags = db::get_tags(conn)?;

    let skill_map = build_id_name_map(&skills, |s| (s.id.clone(), s.name.clone()));
    let attr_map = build_id_name_map(&attributes, |a| (a.id.clone(), a.name.clone()));
    let tag_map = build_id_name_map(&tags, |t| (t.id.clone(), t.name.clone()));

    let mut output: Vec<QuestOutput> = Vec::new();

    for item in &items {
        let quest_output = match item.item_type.as_str() {
            "quest" => {
                if let Some(ref quest) = item.quest {
                    quest_to_output(
                        quest, "quest", item.dismissed_today,
                        None, None,
                        &skill_map, &attr_map, &tag_map,
                    )
                } else {
                    continue;
                }
            }
            "saga" => {
                if let Some(ref slot) = item.saga_slot {
                    let mut qo = quest_to_output(
                        &slot.step, "saga", item.dismissed_today,
                        Some(&slot.saga_id), Some(&slot.saga_name),
                        &skill_map, &attr_map, &tag_map,
                    );
                    // Use saga-level due status and sort order
                    qo.is_due = slot.is_saga_due;
                    qo.sort_order = slot.sort_order;
                    qo
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        // Apply filters
        if filter_active && !quest_output.active {
            continue;
        }
        if filter_due && !quest_output.is_due {
            continue;
        }
        if let Some(ref diff) = filter_difficulty {
            if quest_output.difficulty != *diff {
                continue;
            }
        }

        output.push(quest_output);
    }

    serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
}

fn list_sagas(conn: &nq_core::rusqlite::Connection) -> Result<String, String> {
    let sagas = db::get_sagas_with_progress(conn)?;

    let output: Vec<SagaOutput> = sagas
        .iter()
        .map(|s| SagaOutput {
            id: s.saga.id.clone(),
            name: s.saga.name.clone(),
            cycle_days: s.saga.cycle_days,
            active: s.saga.active,
            is_due: s.is_due,
            total_steps: s.total_steps,
            completed_steps: s.completed_steps,
            created_at: s.saga.created_at.clone(),
            last_run_completed_at: s.saga.last_run_completed_at.clone(),
        })
        .collect();

    serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
}

fn list_tags(conn: &nq_core::rusqlite::Connection) -> Result<String, String> {
    let tags = db::get_tags(conn)?;

    let output: Vec<TagOutput> = tags
        .iter()
        .map(|t| TagOutput {
            id: t.id.clone(),
            name: t.name.clone(),
        })
        .collect();

    serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
}

fn list_skills(conn: &nq_core::rusqlite::Connection) -> Result<String, String> {
    let skills = db::get_skills(conn)?;
    let attributes = db::get_attributes(conn)?;
    let attr_map = build_id_name_map(&attributes, |a| (a.id.clone(), a.name.clone()));

    let output: Vec<SkillOutput> = skills
        .iter()
        .map(|s| SkillOutput {
            id: s.id.clone(),
            name: s.name.clone(),
            attribute: s.attribute_id.as_ref().and_then(|id| attr_map.get(id).cloned()),
            xp: s.xp,
            level: s.level,
            xp_for_current_level: s.xp_for_current_level,
            xp_into_current_level: s.xp_into_current_level,
        })
        .collect();

    serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
}

fn list_attributes(conn: &nq_core::rusqlite::Connection) -> Result<String, String> {
    let attributes = db::get_attributes(conn)?;

    let output: Vec<AttributeOutput> = attributes
        .iter()
        .map(|a| AttributeOutput {
            id: a.id.clone(),
            name: a.name.clone(),
            xp: a.xp,
            level: a.level,
            xp_for_current_level: a.xp_for_current_level,
            xp_into_current_level: a.xp_into_current_level,
        })
        .collect();

    serde_json::to_string_pretty(&output).map_err(|e| e.to_string())
}

fn parse_comma_list(input: Option<String>) -> Vec<String> {
    match input {
        None => Vec::new(),
        Some(s) => s.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
    }
}

fn cmd_add_quest(
    conn: &nq_core::rusqlite::Connection,
    title: String,
    difficulty: String,
    quest_type: String,
    importance: i32,
    cycle_days: Option<i32>,
    time_of_day: Option<String>,
    days_of_week: Option<String>,
    tags: Option<String>,
    skills: Option<String>,
    attributes: Option<String>,
) -> Result<String, String> {
    // Validate enums
    let difficulty = db::Difficulty::try_from_str(&difficulty)?;
    let quest_type = db::QuestType::try_from_str(&quest_type)?;

    // Validate importance
    if importance < 0 || importance > 5 {
        return Err(format!("Invalid importance {}. Must be 0-5", importance));
    }

    // Validate cycle_days for recurring
    if quest_type == db::QuestType::Recurring && cycle_days.is_none() {
        return Err("--cycle-days is required for recurring quests".to_string());
    }

    // Parse bitmasks
    let tod = match time_of_day {
        Some(ref s) => db::parse_time_of_day(s)?,
        None => 15, // anytime
    };
    let dow = match days_of_week {
        Some(ref s) => db::parse_days_of_week(s)?,
        None => 127, // everyday
    };

    let new_quest = db::NewQuest {
        title: title.clone(),
        quest_type,
        cycle_days,
        difficulty,
        time_of_day: tod,
        days_of_week: dow,
        importance,
    };

    let tag_names = parse_comma_list(tags);
    let skill_names = parse_comma_list(skills);
    let attr_names = parse_comma_list(attributes);

    let created = db::add_quest_full(conn, new_quest, tag_names, skill_names, attr_names)?;

    // Resolve IDs back to names for confirmation output
    let all_skills = db::get_skills(conn)?;
    let all_attrs = db::get_attributes(conn)?;
    let all_tags = db::get_tags(conn)?;
    let skill_map = build_id_name_map(&all_skills, |s| (s.id.clone(), s.name.clone()));
    let attr_map = build_id_name_map(&all_attrs, |a| (a.id.clone(), a.name.clone()));
    let tag_map = build_id_name_map(&all_tags, |t| (t.id.clone(), t.name.clone()));

    let result = serde_json::json!({
        "ok": true,
        "id": created.id,
        "title": created.title,
        "skills": resolve_names(&created.skill_ids, &skill_map),
        "attributes": resolve_names(&created.attribute_ids, &attr_map),
        "tags": resolve_names(&created.tag_ids, &tag_map),
    });

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}
