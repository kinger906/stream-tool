use crate::capture;
use crate::mediamtx::MediaMtxState;
use crate::models::{AppSettings, CaptureDevices, StreamTaskInfo, SystemInfo};
use crate::stream_manager::StreamManager;
use tauri::State;

#[tauri::command]
pub fn get_system_info(
    manager: State<'_, StreamManager>,
    mediamtx: State<'_, MediaMtxState>,
    app: tauri::AppHandle,
) -> SystemInfo {
    manager.system_info(&mediamtx, &app)
}

#[tauri::command]
pub fn list_streams(manager: State<'_, StreamManager>) -> Vec<StreamTaskInfo> {
    manager.list_streams()
}

#[tauri::command]
pub fn add_streams(
    paths: Vec<String>,
    manager: State<'_, StreamManager>,
) -> Result<Vec<StreamTaskInfo>, String> {
    manager.add_files(paths)
}

#[tauri::command]
pub async fn list_capture_devices(app: tauri::AppHandle) -> Result<CaptureDevices, String> {
    capture::list_devices(&app).await
}

#[tauri::command]
pub fn add_camera_stream(
    video_device: String,
    audio_device: Option<String>,
    manager: State<'_, StreamManager>,
) -> Result<StreamTaskInfo, String> {
    manager.add_camera_stream(video_device, audio_device)
}

#[tauri::command]
pub fn add_display_stream(
    audio_device: Option<String>,
    manager: State<'_, StreamManager>,
) -> Result<StreamTaskInfo, String> {
    manager.add_display_stream(audio_device)
}

#[tauri::command]
pub fn start_stream(
    id: String,
    manager: State<'_, StreamManager>,
    mediamtx: State<'_, MediaMtxState>,
    app: tauri::AppHandle,
) -> Result<StreamTaskInfo, String> {
    manager.start_stream(&app, &mediamtx, &id)
}

#[tauri::command]
pub fn stop_stream(id: String, manager: State<'_, StreamManager>) -> Result<StreamTaskInfo, String> {
    manager.stop_stream(&id)
}

#[tauri::command]
pub fn start_all_streams(
    manager: State<'_, StreamManager>,
    mediamtx: State<'_, MediaMtxState>,
    app: tauri::AppHandle,
) -> Result<Vec<StreamTaskInfo>, String> {
    manager.start_all(&app, &mediamtx)
}

#[tauri::command]
pub fn stop_all_streams(manager: State<'_, StreamManager>) -> Vec<StreamTaskInfo> {
    manager.stop_all()
}

#[tauri::command]
pub fn remove_stream(id: String, manager: State<'_, StreamManager>) -> Result<(), String> {
    manager.remove_stream(&id)
}

#[tauri::command]
pub fn update_stream(
    id: String,
    loop_enabled: Option<bool>,
    copy_mode: Option<bool>,
    manager: State<'_, StreamManager>,
) -> Result<StreamTaskInfo, String> {
    manager.update_stream(&id, loop_enabled, copy_mode)
}

#[tauri::command]
pub fn update_settings(
    max_concurrent: usize,
    manager: State<'_, StreamManager>,
) -> AppSettings {
    manager.set_max_concurrent(max_concurrent)
}

#[tauri::command]
pub fn start_mediamtx(
    mediamtx: State<'_, MediaMtxState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    crate::mediamtx::start(&app, &mediamtx)
}
