use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

fn list_gifs(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "gif").unwrap_or(false) {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    files.push(name.to_string());
                }
            }
        }
    }
    files.sort();
    files
}

fn main() {
    let ui_images = Path::new("../ui/images");

    let categories = [
        ("quest-givers", "quest-givers"),
        ("lane1", "lane1"),
        ("lane2", "lane2"),
        ("lane3", "lane3"),
        ("monsters", "monsters"),
        ("victory", "victory"),
        ("defeat", "defeat"),
    ];

    let mut manifest: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (key, folder) in &categories {
        let dir = ui_images.join(folder);
        let gifs = list_gifs(&dir)
            .iter()
            .map(|f| format!("images/{}/{}", folder, f))
            .collect();
        manifest.insert(key, gifs);

        // Tell cargo to re-run build.rs if the image folder contents change
        println!("cargo:rerun-if-changed=../ui/images/{}", folder);
    }

    let json = serde_json::to_string_pretty(&manifest).expect("Failed to serialize manifest");
    fs::write(ui_images.join("manifest.json"), json).expect("Failed to write manifest.json");

    tauri_build::build()
}
