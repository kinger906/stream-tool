use crate::models::ScenePreset;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

fn scenes_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("scenes");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("scenes.json"))
}

pub fn list_scenes(app: &AppHandle) -> Result<Vec<ScenePreset>, String> {
    let path = scenes_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn save_scenes(app: &AppHandle, scenes: &[ScenePreset]) -> Result<(), String> {
    let path = scenes_path(app)?;
    let text = serde_json::to_string_pretty(scenes).map_err(|e| e.to_string())?;
    fs::write(path, text).map_err(|e| e.to_string())
}

pub fn upsert_scene(app: &AppHandle, mut scene: ScenePreset) -> Result<ScenePreset, String> {
    if scene.name.trim().is_empty() {
        return Err("场景名称不能为空".to_string());
    }
    if scene.id.is_empty() {
        scene.id = Uuid::new_v4().simple().to_string()[..8].to_string();
    }
    if scene.created_at.is_empty() {
        scene.created_at = chrono_like_now();
    }
    let mut scenes = list_scenes(app)?;
    if let Some(existing) = scenes.iter_mut().find(|s| s.id == scene.id) {
        *existing = scene.clone();
    } else {
        scenes.insert(0, scene.clone());
    }
    save_scenes(app, &scenes)?;
    Ok(scene)
}

pub fn delete_scene(app: &AppHandle, id: &str) -> Result<(), String> {
    let mut scenes = list_scenes(app)?;
    scenes.retain(|s| s.id != id);
    save_scenes(app, &scenes)
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}
