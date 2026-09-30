use crate::models::{AppSettings, InternalStreamTask};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

pub struct MediaMtxState {
    pub child: Mutex<Option<CommandChild>>,
    pub config_path: Mutex<Option<PathBuf>>,
}

impl MediaMtxState {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            config_path: Mutex::new(None),
        }
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().unwrap().is_some()
    }
}

pub fn record_dir(app: &AppHandle, settings: &AppSettings) -> PathBuf {
    if let Some(dir) = &settings.record_dir {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("recordings")
}

pub fn write_config(
    app: &AppHandle,
    tasks: &HashMap<String, InternalStreamTask>,
    settings: &AppSettings,
) -> Result<PathBuf, String> {
    let config_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("mediamtx");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let target = config_dir.join("mediamtx.yml");

    let rec_dir = record_dir(app, settings);
    fs::create_dir_all(&rec_dir).map_err(|e| e.to_string())?;
    let rec_dir_str = rec_dir.to_string_lossy().replace('\\', "/");

    let mut path_block = String::new();
    for task in tasks.values() {
        path_block.push_str(&format!("  {}:\n", task.stream_name));
        if task.record_enabled {
            path_block.push_str("    record: yes\n");
            path_block.push_str(&format!(
                "    recordPath: {}/{}-%Y-%m-%d_%H-%M-%S\n",
                rec_dir_str, task.stream_name
            ));
        }
        if let (Some(user), Some(pass)) = (&settings.rtsp_username, &settings.rtsp_password) {
            if !user.is_empty() && !pass.is_empty() {
                path_block.push_str(&format!("    readUser: {user}\n"));
                path_block.push_str(&format!("    readPass: {pass}\n"));
            }
        }
    }
    path_block.push_str("  all_others:\n");

    let yaml = format!(
        r#"logLevel: info
logDestinations: [stdout]

rtsp: yes
rtspAddress: :8554

hls: yes
hlsAddress: :8888
hlsAllowOrigin: '*'
hlsAlwaysRemux: yes
hlsVariant: mpegts
hlsSegmentCount: 7
hlsSegmentDuration: 1s

webrtc: yes
webrtcAddress: :8889
webrtcAllowOrigin: '*'

rtmp: no
srt: no

paths:
{path_block}"#
    );

    fs::write(&target, yaml).map_err(|e| e.to_string())?;
    Ok(target)
}

pub fn restart(app: &AppHandle, state: &MediaMtxState) -> Result<(), String> {
    stop(state);
    start(app, state)
}

pub fn check_sidecar_available(app: &AppHandle) -> bool {
    app.shell().sidecar("mediamtx").is_ok()
}

pub fn dependency_status(app: &AppHandle) -> crate::models::DependencyStatus {
    let ffmpeg_available = app.shell().sidecar("ffmpeg").is_ok();
    let mediamtx_available = app.shell().sidecar("mediamtx").is_ok();

    crate::models::DependencyStatus {
        ffmpeg_available,
        mediamtx_available,
        ffmpeg_path_hint: crate::ffmpeg::sidecar_target_hint("ffmpeg"),
        mediamtx_path_hint: crate::ffmpeg::sidecar_target_hint("mediamtx"),
    }
}

pub fn start(app: &AppHandle, state: &MediaMtxState) -> Result<(), String> {
    if state.is_running() {
        return Ok(());
    }

    if !check_sidecar_available(app) {
        return Err(format!(
            "未找到 MediaMTX sidecar，请将 mediamtx 放入 {}",
            crate::ffmpeg::sidecar_target_hint("mediamtx")
        ));
    }

    let config_path = state
        .config_path
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "MediaMTX 配置未初始化".to_string())?;

    let sidecar = app.shell().sidecar("mediamtx").map_err(|e| e.to_string())?;
    let (mut rx, child) = sidecar
        .args([config_path.to_string_lossy().to_string()])
        .spawn()
        .map_err(|e| e.to_string())?;

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let tauri_plugin_shell::process::CommandEvent::Terminated(payload) = event {
                if payload.code.unwrap_or(0) != 0 {
                    let _ = app_handle.emit("mediamtx-stopped", payload.code);
                }
                break;
            }
        }
    });

    *state.child.lock().unwrap() = Some(child);
    Ok(())
}

pub fn stop(state: &MediaMtxState) {
    if let Some(child) = state.child.lock().unwrap().take() {
        let _ = child.kill();
    }
}

pub fn write_config_only(
    app: &AppHandle,
    state: &MediaMtxState,
    tasks: &HashMap<String, InternalStreamTask>,
    settings: &AppSettings,
) -> Result<(), String> {
    let path = write_config(app, tasks, settings)?;
    *state.config_path.lock().unwrap() = Some(path);
    Ok(())
}

pub fn sync_config_and_restart(
    app: &AppHandle,
    state: &MediaMtxState,
    tasks: &HashMap<String, InternalStreamTask>,
    settings: &AppSettings,
) -> Result<(), String> {
    write_config_only(app, state, tasks, settings)?;
    if state.is_running() {
        restart(app, state)?;
        // Give MediaMTX a moment to bind ports before publishers reconnect.
        std::thread::sleep(std::time::Duration::from_millis(400));
    }
    Ok(())
}

pub fn init_config(
    app: &AppHandle,
    state: &MediaMtxState,
    tasks: &HashMap<String, InternalStreamTask>,
    settings: &AppSettings,
) -> Result<(), String> {
    let path = write_config(app, tasks, settings)?;
    *state.config_path.lock().unwrap() = Some(path);
    Ok(())
}
