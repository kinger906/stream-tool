use crate::models::DependencyStatus;
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

pub fn prepare_config(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("resources")
        .join("mediamtx.yml");

    let config_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("mediamtx");
    fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    let target = config_dir.join("mediamtx.yml");

    if resource_path.exists() {
        fs::copy(&resource_path, &target).map_err(|e| e.to_string())?;
    } else {
        let fallback = include_str!("../resources/mediamtx.yml");
        fs::write(&target, fallback).map_err(|e| e.to_string())?;
    }

    Ok(target)
}

pub fn check_sidecar_available(app: &AppHandle) -> bool {
    app.shell().sidecar("mediamtx").is_ok()
}

pub fn dependency_status(app: &AppHandle) -> DependencyStatus {
    let ffmpeg_available = app.shell().sidecar("ffmpeg").is_ok();
    let mediamtx_available = app.shell().sidecar("mediamtx").is_ok();

    DependencyStatus {
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

    let config_path = prepare_config(app)?;
    *state.config_path.lock().unwrap() = Some(config_path.clone());

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
