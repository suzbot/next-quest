pub mod db;

// Re-export dependencies so consumers don't need them as direct dependencies.
pub use rusqlite;
pub use dirs;

use std::path::PathBuf;

/// Returns the path to the Next Quest database.
/// Creates the parent directory if it doesn't exist.
pub fn db_path() -> PathBuf {
    let dir = dirs::data_dir()
        .expect("Could not find data directory")
        .join("com.nextquest.desktop");
    std::fs::create_dir_all(&dir).expect("Failed to create data directory");
    dir.join("next-quest.db")
}
