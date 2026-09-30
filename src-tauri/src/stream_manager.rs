use crate::ffmpeg::{
    build_probe_args, build_stream_args, parse_duration_from_stderr, parse_elapsed_from_stderr,
    parse_stats_from_stderr, summarize_error,
};
use crate::mediamtx::{self, MediaMtxState};
use crate::models::{
    slug_from_filename, validate_rtmp_url, validate_stream_name, AppSettings, CaptureRegion,
    InternalStreamTask, QualityPreset, ScenePreset, SceneStreamSpec, SourceType, StreamStatus,
    StreamTaskInfo, SystemInfo,
};
use crate::scenes;
use crate::network;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use uuid::Uuid;

struct RunningProcess {
    child: CommandChild,
    stderr: Arc<Mutex<String>>,
}

pub struct StreamManager {
    tasks: Mutex<HashMap<String, InternalStreamTask>>,
    processes: Mutex<HashMap<String, RunningProcess>>,
    settings: Mutex<AppSettings>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            processes: Mutex::new(HashMap::new()),
            settings: Mutex::new(AppSettings::default()),
        }
    }

    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn lan_ip(&self) -> String {
        let settings = self.settings.lock().unwrap();
        network::resolve_lan_ip(&settings.selected_lan_ip)
    }

    pub fn list_lan_ips(&self) -> Vec<String> {
        network::list_lan_ips()
    }

    fn sync_mediamtx(&self, app: &AppHandle, mediamtx: &MediaMtxState) -> Result<(), String> {
        let tasks = self.tasks.lock().unwrap().clone();
        let settings = self.settings.lock().unwrap().clone();
        mediamtx::sync_config_and_restart(app, mediamtx, &tasks, &settings)
    }

    /// Update MediaMTX config on disk without restarting (paths are covered by all_others).
    fn write_mediamtx_config(&self, app: &AppHandle, mediamtx: &MediaMtxState) -> Result<(), String> {
        let tasks = self.tasks.lock().unwrap().clone();
        let settings = self.settings.lock().unwrap().clone();
        mediamtx::write_config_only(app, mediamtx, &tasks, &settings)
    }

    pub fn init_mediamtx_config(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
    ) -> Result<(), String> {
        let tasks = self.tasks.lock().unwrap().clone();
        let settings = self.settings.lock().unwrap().clone();
        mediamtx::init_config(app, mediamtx, &tasks, &settings)
    }

    pub fn list_streams(&self) -> Vec<StreamTaskInfo> {
        let settings = self.settings.lock().unwrap().clone();
        let lan_ip = self.lan_ip();
        self.tasks
            .lock()
            .unwrap()
            .values()
            .map(|task| task.to_info(&lan_ip, &settings))
            .collect()
    }

    fn next_id(&self) -> String {
        Uuid::new_v4().simple().to_string()[..8].to_string()
    }

    fn ensure_unique_stream_name(&self, base: &str) -> String {
        let mut name = base.to_string();
        if validate_stream_name(&name).is_err() {
            name = "stream".to_string();
        }
        let tasks = self.tasks.lock().unwrap();
        if !tasks.values().any(|t| t.stream_name == name) {
            return name;
        }
        let stem: String = name.chars().take(28).collect();
        for i in 2..1000 {
            let candidate = format!("{stem}_{i}");
            if validate_stream_name(&candidate).is_ok()
                && !tasks.values().any(|t| t.stream_name == candidate)
            {
                return candidate;
            }
        }
        format!("s_{}", &Uuid::new_v4().simple().to_string()[..6])
    }

    fn insert_task(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        task: InternalStreamTask,
    ) -> Result<StreamTaskInfo, String> {
        let settings = self.settings.lock().unwrap().clone();
        let lan_ip = self.lan_ip();
        let info = task.to_info(&lan_ip, &settings);
        self.tasks.lock().unwrap().insert(task.id.clone(), task);
        // New stream names are accepted via all_others; avoid restarting MediaMTX
        // (restart would drop other live publishers).
        self.write_mediamtx_config(app, mediamtx)?;
        Ok(info)
    }

    fn new_task_base(
        &self,
        filename: &str,
        path: PathBuf,
        source_type: SourceType,
        loop_enabled: bool,
        copy_mode: bool,
        audio_device: Option<String>,
        window_title: Option<String>,
    ) -> InternalStreamTask {
        let stream_name = self.ensure_unique_stream_name(&slug_from_filename(filename));
        let using_transcode = source_type != SourceType::File || !copy_mode;
        let auto_reconnect = self.settings.lock().unwrap().auto_reconnect_default;
        let file_size_bytes = crate::models::read_file_size_bytes(&path, &source_type);
        InternalStreamTask {
            id: self.next_id(),
            stream_name,
            path,
            filename: filename.to_string(),
            source_type,
            status: StreamStatus::Idle,
            loop_enabled,
            copy_mode,
            record_enabled: false,
            auto_reconnect,
            quality_preset: QualityPreset::default(),
            audio_device,
            window_title,
            region: None,
            rtmp_url: None,
            error: None,
            elapsed_secs: 0.0,
            duration_secs: None,
            file_size_bytes,
            bitrate_kbps: None,
            fps: None,
            using_transcode,
            reconnect_attempts: 0,
            user_stopped: false,
        }
    }

    pub fn add_region_stream(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        region: CaptureRegion,
        audio_device: Option<String>,
    ) -> Result<StreamTaskInfo, String> {
        region.validate()?;
        let label = if let Some(audio) = &audio_device {
            format!(
                "区域 {}x{}@{},{} + {audio}",
                region.width, region.height, region.x, region.y
            )
        } else {
            format!(
                "区域 {}x{}@{},{}",
                region.width, region.height, region.x, region.y
            )
        };
        let mut task = self.new_task_base(
            &label,
            PathBuf::from("region"),
            SourceType::Region,
            false,
            false,
            audio_device,
            None,
        );
        task.region = Some(region);
        task.using_transcode = true;
        self.insert_task(app, mediamtx, task)
    }

    pub fn add_files(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        paths: Vec<String>,
    ) -> Result<Vec<StreamTaskInfo>, String> {
        let mut added = Vec::new();

        for path_str in paths {
            let path = PathBuf::from(&path_str);
            if !path.exists() {
                continue;
            }

            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path_str.clone());

            let task = self.new_task_base(
                &filename,
                path,
                SourceType::File,
                true,
                true,
                None,
                None,
            );
            added.push(self.insert_task(app, mediamtx, task)?);
        }

        Ok(added)
    }

    /// Add local video files and start streaming immediately — for large-file
    /// playback on weak terminals (RTSP/HLS instead of opening the whole file).
    pub fn add_and_start_files(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        paths: Vec<String>,
    ) -> Result<Vec<StreamTaskInfo>, String> {
        let added = self.add_files(app, mediamtx, paths)?;
        if added.is_empty() {
            return Err("未找到有效的视频文件".to_string());
        }

        let mut started = Vec::new();
        let mut errors = Vec::new();
        for info in &added {
            match self.start_stream(app, mediamtx, &info.id) {
                Ok(s) => started.push(s),
                Err(err) => {
                    errors.push(format!("{}: {err}", info.filename));
                    started.push(info.clone());
                }
            }
        }

        if !errors.is_empty() && started.iter().all(|s| s.status != StreamStatus::Running) {
            return Err(errors.join("\n"));
        }

        Ok(started)
    }

    pub fn add_camera_stream(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        video_device: String,
        audio_device: Option<String>,
    ) -> Result<StreamTaskInfo, String> {
        if video_device.trim().is_empty() {
            return Err("请选择摄像头".to_string());
        }

        let label = if let Some(audio) = &audio_device {
            format!("摄像头: {video_device} + {audio}")
        } else {
            format!("摄像头: {video_device}")
        };

        let mut task = self.new_task_base(
            &label,
            PathBuf::from(video_device),
            SourceType::Camera,
            false,
            false,
            audio_device,
            None,
        );
        task.using_transcode = true;
        self.insert_task(app, mediamtx, task)
    }

    pub fn add_display_stream(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        audio_device: Option<String>,
    ) -> Result<StreamTaskInfo, String> {
        let label = if let Some(audio) = &audio_device {
            format!("桌面采集 + {audio}")
        } else {
            "桌面采集".to_string()
        };

        let mut task = self.new_task_base(
            &label,
            PathBuf::from("desktop"),
            SourceType::Display,
            false,
            false,
            audio_device,
            None,
        );
        task.using_transcode = true;
        self.insert_task(app, mediamtx, task)
    }

    pub fn add_window_stream(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        window_title: String,
        audio_device: Option<String>,
    ) -> Result<StreamTaskInfo, String> {
        if window_title.trim().is_empty() {
            return Err("请选择窗口".to_string());
        }

        let label = if let Some(audio) = &audio_device {
            format!("窗口: {window_title} + {audio}")
        } else {
            format!("窗口: {window_title}")
        };

        let mut task = self.new_task_base(
            &label,
            PathBuf::from("window"),
            SourceType::Window,
            false,
            false,
            audio_device,
            Some(window_title),
        );
        task.using_transcode = true;
        self.insert_task(app, mediamtx, task)
    }

    pub fn remove_stream(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        id: &str,
    ) -> Result<(), String> {
        self.stop_stream_internal(id)?;
        self.tasks.lock().unwrap().remove(id);
        self.write_mediamtx_config(app, mediamtx)?;
        Ok(())
    }

    pub fn update_stream(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        id: &str,
        loop_enabled: Option<bool>,
        copy_mode: Option<bool>,
        stream_name: Option<String>,
        record_enabled: Option<bool>,
        quality_preset: Option<QualityPreset>,
        auto_reconnect: Option<bool>,
        rtmp_url: Option<String>,
        region: Option<CaptureRegion>,
    ) -> Result<StreamTaskInfo, String> {
        if let Some(ref name) = stream_name {
            validate_stream_name(name)?;
            let taken = self
                .tasks
                .lock()
                .unwrap()
                .values()
                .any(|t| t.id != id && t.stream_name == *name);
            if taken {
                return Err("流名称已被占用".to_string());
            }
        }
        if let Some(ref url) = rtmp_url {
            validate_rtmp_url(url)?;
        }
        if let Some(ref r) = region {
            r.validate()?;
        }

        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(id).ok_or_else(|| "任务不存在".to_string())?;

        if task.status == StreamStatus::Running || task.status == StreamStatus::Reconnecting {
            return Err("推流进行中，无法修改设置".to_string());
        }

        if let Some(v) = loop_enabled {
            if task.source_type != SourceType::File {
                return Err("仅文件推流支持循环设置".to_string());
            }
            task.loop_enabled = v;
        }
        if let Some(v) = copy_mode {
            if task.source_type != SourceType::File {
                return Err("仅文件推流支持 Copy 设置".to_string());
            }
            task.copy_mode = v;
            task.using_transcode = !v;
        }
        if let Some(name) = stream_name {
            task.stream_name = name;
        }
        if let Some(v) = record_enabled {
            task.record_enabled = v;
        }
        if let Some(v) = quality_preset {
            task.quality_preset = v;
        }
        if let Some(v) = auto_reconnect {
            task.auto_reconnect = v;
        }
        if let Some(url) = rtmp_url {
            task.rtmp_url = if url.trim().is_empty() {
                None
            } else {
                Some(url.trim().to_string())
            };
        }
        if let Some(r) = region {
            if task.source_type != SourceType::Region {
                return Err("仅区域采集支持修改截取区域".to_string());
            }
            task.region = Some(r);
        }

        let settings = self.settings.lock().unwrap().clone();
        let lan_ip = self.lan_ip();
        let info = task.to_info(&lan_ip, &settings);
        let needs_restart = record_enabled.is_some();
        drop(tasks);

        if needs_restart {
            self.sync_mediamtx(app, mediamtx)?;
        } else {
            self.write_mediamtx_config(app, mediamtx)?;
        }
        Ok(info)
    }

    pub fn update_settings(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        max_concurrent: Option<usize>,
        selected_lan_ip: Option<String>,
        rtsp_username: Option<String>,
        rtsp_password: Option<String>,
        record_dir: Option<String>,
        auto_reconnect_default: Option<bool>,
        public_base_url: Option<String>,
        minimize_to_tray: Option<bool>,
    ) -> Result<AppSettings, String> {
        {
            let mut settings = self.settings.lock().unwrap();
            if let Some(v) = max_concurrent {
                settings.max_concurrent = v.clamp(1, 32);
            }
            if let Some(v) = selected_lan_ip {
                settings.selected_lan_ip = if v.is_empty() { None } else { Some(v) };
            }
            if let Some(v) = rtsp_username {
                settings.rtsp_username = if v.is_empty() { None } else { Some(v) };
            }
            if let Some(v) = rtsp_password {
                settings.rtsp_password = if v.is_empty() { None } else { Some(v) };
            }
            if let Some(v) = record_dir {
                settings.record_dir = if v.is_empty() { None } else { Some(v) };
            }
            if let Some(v) = auto_reconnect_default {
                settings.auto_reconnect_default = v;
            }
            if let Some(v) = public_base_url {
                settings.public_base_url = if v.trim().is_empty() {
                    None
                } else {
                    Some(v.trim().trim_end_matches('/').to_string())
                };
            }
            if let Some(v) = minimize_to_tray {
                settings.minimize_to_tray = v;
            }
        }
        self.sync_mediamtx(app, mediamtx)?;
        Ok(self.get_settings())
    }

    pub fn save_current_scene(
        &self,
        app: &AppHandle,
        name: String,
    ) -> Result<ScenePreset, String> {
        let streams: Vec<SceneStreamSpec> = self
            .tasks
            .lock()
            .unwrap()
            .values()
            .map(|t| t.to_scene_spec())
            .collect();
        if streams.is_empty() {
            return Err("当前没有可保存的推流任务".to_string());
        }
        let scene = ScenePreset {
            id: String::new(),
            name,
            created_at: String::new(),
            streams,
        };
        scenes::upsert_scene(app, scene)
    }

    pub fn list_scenes(&self, app: &AppHandle) -> Result<Vec<ScenePreset>, String> {
        scenes::list_scenes(app)
    }

    pub fn delete_scene(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        scenes::delete_scene(app, id)
    }

    pub fn apply_scene(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        scene_id: &str,
        replace: bool,
    ) -> Result<Vec<StreamTaskInfo>, String> {
        let scenes_list = scenes::list_scenes(app)?;
        let scene = scenes_list
            .into_iter()
            .find(|s| s.id == scene_id)
            .ok_or_else(|| "场景不存在".to_string())?;

        if replace {
            let ids: Vec<String> = self.tasks.lock().unwrap().keys().cloned().collect();
            for id in ids {
                let _ = self.remove_stream(app, mediamtx, &id);
            }
        }

        let mut added = Vec::new();
        for spec in scene.streams {
            let task = self.task_from_scene_spec(spec)?;
            added.push(self.insert_task(app, mediamtx, task)?);
        }
        Ok(added)
    }

    fn task_from_scene_spec(&self, spec: SceneStreamSpec) -> Result<InternalStreamTask, String> {
        validate_stream_name(&spec.stream_name)?;
        if let Some(ref url) = spec.rtmp_url {
            validate_rtmp_url(url)?;
        }
        if let Some(ref r) = spec.region {
            r.validate()?;
        }
        let stream_name = self.ensure_unique_stream_name(&spec.stream_name);
        let path = PathBuf::from(&spec.path);
        let file_size_bytes = crate::models::read_file_size_bytes(&path, &spec.source_type);
        let using_transcode = spec.source_type != SourceType::File || !spec.copy_mode;
        Ok(InternalStreamTask {
            id: self.next_id(),
            stream_name,
            path,
            filename: spec.filename,
            source_type: spec.source_type,
            status: StreamStatus::Idle,
            loop_enabled: spec.loop_enabled,
            copy_mode: spec.copy_mode,
            record_enabled: spec.record_enabled,
            auto_reconnect: spec.auto_reconnect,
            quality_preset: spec.quality_preset,
            audio_device: spec.audio_device,
            window_title: spec.window_title,
            region: spec.region,
            rtmp_url: spec.rtmp_url.filter(|s| !s.trim().is_empty()),
            error: None,
            elapsed_secs: 0.0,
            duration_secs: None,
            file_size_bytes,
            bitrate_kbps: None,
            fps: None,
            using_transcode,
            reconnect_attempts: 0,
            user_stopped: false,
        })
    }

    pub fn start_stream(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        id: &str,
    ) -> Result<StreamTaskInfo, String> {
        if !app.shell().sidecar("ffmpeg").is_ok() {
            return Err(format!(
                "未找到 FFmpeg sidecar，请运行 npm run fetch-sidecars 或手动放入 {}",
                crate::ffmpeg::sidecar_target_hint("ffmpeg")
            ));
        }

        // Keep MediaMTX up when possible. Restart only if this stream needs
        // path-level options (recording) that require a config reload.
        let needs_restart = self
            .tasks
            .lock()
            .unwrap()
            .get(id)
            .map(|t| t.record_enabled)
            .unwrap_or(false);
        if needs_restart {
            self.sync_mediamtx(app, mediamtx)?;
        } else {
            self.write_mediamtx_config(app, mediamtx)?;
        }

        if !mediamtx.is_running() {
            mediamtx::start(app, mediamtx)?;
            std::thread::sleep(std::time::Duration::from_millis(400));
        }

        let running_count = self.processes.lock().unwrap().len();
        let max = self.settings.lock().unwrap().max_concurrent;
        if running_count >= max {
            return Err(format!("已达并发上限 ({max})，请先停止其它推流或提高上限"));
        }

        if self.processes.lock().unwrap().contains_key(id) {
            return Err("该任务已在推流中".to_string());
        }

        let task_snapshot = {
            let mut tasks = self.tasks.lock().unwrap();
            let task = tasks.get_mut(id).ok_or_else(|| "任务不存在".to_string())?;
            task.status = StreamStatus::Running;
            task.error = None;
            task.elapsed_secs = 0.0;
            task.bitrate_kbps = None;
            task.fps = None;
            task.user_stopped = false;
            task.reconnect_attempts = 0;
            if task.source_type == SourceType::File {
                task.using_transcode = !task.copy_mode
                    || task
                        .rtmp_url
                        .as_ref()
                        .map(|s| !s.trim().is_empty())
                        .unwrap_or(false);
            } else {
                task.using_transcode = true;
            }
            task.clone_snapshot()
        };

        let app_clone = app.clone();
        let id_owned = id.to_string();

        match self.spawn_ffmpeg(app.clone(), &id_owned, &task_snapshot, move |stream_id, stderr, exit_code| {
            Self::handle_process_exit(&app_clone, &stream_id, &stderr, exit_code);
        }) {
            Ok(()) => {}
            Err(err) => {
                let mut tasks = self.tasks.lock().unwrap();
                if let Some(task) = tasks.get_mut(id) {
                    task.status = StreamStatus::Error;
                    task.error = Some(err.clone());
                }
                return Err(err);
            }
        }

        if task_snapshot.source_type == SourceType::File {
            self.probe_duration_async(app.clone(), id.to_string(), task_snapshot.path);
        }

        let settings = self.settings.lock().unwrap().clone();
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .get(id)
            .map(|t| t.to_info(&self.lan_ip(), &settings))
            .unwrap())
    }

    fn handle_process_exit(
        app: &AppHandle,
        id: &str,
        stderr: &str,
        exit_code: Option<i32>,
    ) {
        let manager = app.state::<StreamManager>();
        manager.processes.lock().unwrap().remove(id);

        let (user_stopped, auto_reconnect, should_retry_transcode, attempts) = {
            let tasks = manager.tasks.lock().unwrap();
            tasks
                .get(id)
                .map(|t| {
                    (
                        t.user_stopped,
                        t.auto_reconnect,
                        t.source_type == SourceType::File && t.copy_mode && !t.using_transcode,
                        t.reconnect_attempts,
                    )
                })
                .unwrap_or((true, false, false, 0))
        };

        if user_stopped {
            let mut tasks = manager.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(id) {
                task.status = StreamStatus::Stopped;
            }
            let _ = app.emit("streams-changed", ());
            return;
        }

        if exit_code.is_none() || exit_code == Some(0) {
            let mut tasks = manager.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(id) {
                if task.source_type == SourceType::File && task.loop_enabled {
                    task.status = StreamStatus::Running;
                } else {
                    task.status = StreamStatus::Stopped;
                }
            }
            let _ = app.emit("streams-changed", ());
            return;
        }

        if should_retry_transcode {
            let task_snapshot = {
                let mut tasks = manager.tasks.lock().unwrap();
                let task = tasks.get_mut(id).unwrap();
                task.using_transcode = true;
                task.status = StreamStatus::Running;
                task.error = Some("Copy 模式失败，正在尝试转码推流…".to_string());
                task.clone_snapshot()
            };

            let app_clone = app.clone();
            let id_owned = id.to_string();

            if manager
                .spawn_ffmpeg(app.clone(), &id_owned, &task_snapshot, move |stream_id, stderr, exit_code| {
                    Self::handle_process_exit(&app_clone, &stream_id, &stderr, exit_code);
                })
                .is_ok()
            {
                let _ = app.emit("streams-changed", ());
                return;
            }
        }

        if auto_reconnect && attempts < 8 {
            let delay_ms = match attempts {
                0 => 2000,
                1 => 4000,
                2 => 8000,
                _ => 12000,
            };
            {
                let mut tasks = manager.tasks.lock().unwrap();
                if let Some(task) = tasks.get_mut(id) {
                    task.reconnect_attempts = attempts + 1;
                    task.status = StreamStatus::Reconnecting;
                    task.error = Some(format!(
                        "推流中断（{}), {} 秒后自动重连… ({}/8)",
                        summarize_error(stderr),
                        delay_ms / 1000,
                        attempts + 1
                    ));
                }
            }
            let _ = app.emit("streams-changed", ());

            let app_clone = app.clone();
            let id_owned = id.to_string();
            tauri::async_runtime::spawn(async move {
                tokio_sleep(delay_ms).await;
                let manager = app_clone.state::<StreamManager>();
                let mediamtx = app_clone.state::<MediaMtxState>();
                let still_wanted = {
                    let tasks = manager.tasks.lock().unwrap();
                    tasks
                        .get(&id_owned)
                        .map(|t| t.auto_reconnect && !t.user_stopped)
                        .unwrap_or(false)
                };
                if !still_wanted {
                    return;
                }
                if let Err(err) = manager.start_stream_reconnect(&app_clone, &mediamtx, &id_owned) {
                    let mut tasks = manager.tasks.lock().unwrap();
                    if let Some(task) = tasks.get_mut(&id_owned) {
                        task.status = StreamStatus::Error;
                        task.error = Some(err);
                    }
                    let _ = app_clone.emit("streams-changed", ());
                }
            });
            return;
        }

        let summary = summarize_error(stderr);
        {
            let mut tasks = manager.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(id) {
                task.status = StreamStatus::Error;
                task.error = Some(summary);
            }
        }
        let _ = app.emit("streams-changed", ());
    }

    /// Restart after disconnect without resetting reconnect_attempts.
    fn start_stream_reconnect(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
        id: &str,
    ) -> Result<(), String> {
        if self.processes.lock().unwrap().contains_key(id) {
            return Ok(());
        }
        if !mediamtx.is_running() {
            mediamtx::start(app, mediamtx)?;
            std::thread::sleep(std::time::Duration::from_millis(400));
        }

        let task_snapshot = {
            let mut tasks = self.tasks.lock().unwrap();
            let task = tasks.get_mut(id).ok_or_else(|| "任务不存在".to_string())?;
            if task.user_stopped {
                return Ok(());
            }
            task.status = StreamStatus::Running;
            task.error = None;
            task.using_transcode = true;
            task.clone_snapshot()
        };

        let app_clone = app.clone();
        let id_owned = id.to_string();
        self.spawn_ffmpeg(app.clone(), &id_owned, &task_snapshot, move |stream_id, stderr, exit_code| {
            Self::handle_process_exit(&app_clone, &stream_id, &stderr, exit_code);
        })?;
        let _ = app.emit("streams-changed", ());
        Ok(())
    }

    fn spawn_ffmpeg<F>(
        &self,
        app: AppHandle,
        id: &str,
        task: &InternalStreamTask,
        on_exit: F,
    ) -> Result<(), String>
    where
        F: FnOnce(String, String, Option<i32>) + Send + 'static,
    {
        let transcode = task.using_transcode || task.source_type != SourceType::File;
        let args = build_stream_args(task, transcode);
        let sidecar = app.shell().sidecar("ffmpeg").map_err(|e| e.to_string())?;
        let (mut rx, child) = sidecar.args(args).spawn().map_err(|e| e.to_string())?;

        let stderr_buf = Arc::new(Mutex::new(String::new()));
        let stderr_for_task = stderr_buf.clone();
        let id_for_task = id.to_string();

        tauri::async_runtime::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stderr(line) => {
                        let chunk = String::from_utf8_lossy(&line).to_string();
                        stderr_for_task.lock().unwrap().push_str(&chunk);
                    }
                    CommandEvent::Terminated(payload) => {
                        let stderr = stderr_for_task.lock().unwrap().clone();
                        on_exit(id_for_task, stderr, payload.code);
                        break;
                    }
                    _ => {}
                }
            }
        });

        self.processes.lock().unwrap().insert(
            id.to_string(),
            RunningProcess {
                child,
                stderr: stderr_buf.clone(),
            },
        );

        let app_for_poll = app.clone();
        let id_for_poll = id.to_string();
        tauri::async_runtime::spawn(async move {
            loop {
                tokio_sleep(1000).await;
                let manager = app_for_poll.state::<StreamManager>();
                let still_running = manager.processes.lock().unwrap().contains_key(&id_for_poll);
                if !still_running {
                    break;
                }

                let snapshot = {
                    let processes = manager.processes.lock().unwrap();
                    processes.get(&id_for_poll).map(|p| {
                        let stderr = p.stderr.lock().unwrap();
                        let elapsed = parse_elapsed_from_stderr(&stderr);
                        let (fps, bitrate) = parse_stats_from_stderr(&stderr);
                        (elapsed, fps, bitrate)
                    })
                };

                if let Some((elapsed, fps, bitrate)) = snapshot {
                    let mut tasks = manager.tasks.lock().unwrap();
                    if let Some(task) = tasks.get_mut(&id_for_poll) {
                        if let Some(e) = elapsed {
                            task.elapsed_secs = e;
                        }
                        if let Some(f) = fps {
                            task.fps = Some(f);
                        }
                        if let Some(b) = bitrate {
                            task.bitrate_kbps = Some(b);
                        }
                    }
                    let _ = app_for_poll.emit("streams-changed", ());
                }
            }
        });

        Ok(())
    }

    fn probe_duration_async(&self, app: AppHandle, id: String, path: PathBuf) {
        tauri::async_runtime::spawn(async move {
            let args = build_probe_args(&path);
            let Ok(sidecar) = app.shell().sidecar("ffmpeg") else {
                return;
            };
            let Ok((mut rx, _child)) = sidecar.args(args).spawn() else {
                return;
            };

            let mut stderr = String::new();
            while let Some(event) = rx.recv().await {
                if let CommandEvent::Stderr(line) = event {
                    stderr.push_str(&String::from_utf8_lossy(&line));
                }
            }

            if let Some(duration) = parse_duration_from_stderr(&stderr) {
                let manager = app.state::<StreamManager>();
                let mut tasks = manager.tasks.lock().unwrap();
                if let Some(task) = tasks.get_mut(&id) {
                    task.duration_secs = Some(duration);
                }
                let _ = app.emit("streams-changed", ());
            }
        });
    }

    pub fn stop_stream(&self, id: &str) -> Result<StreamTaskInfo, String> {
        self.stop_stream_internal(id)?;
        let settings = self.settings.lock().unwrap().clone();
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .get(id)
            .map(|t| t.to_info(&self.lan_ip(), &settings))
            .ok_or_else(|| "任务不存在".to_string())?)
    }

    fn stop_stream_internal(&self, id: &str) -> Result<(), String> {
        {
            let mut tasks = self.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(id) {
                task.user_stopped = true;
                task.reconnect_attempts = 0;
            }
        }

        if let Some(process) = self.processes.lock().unwrap().remove(id) {
            let _ = process.child.kill();
        }

        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(id) {
            if task.status == StreamStatus::Running || task.status == StreamStatus::Reconnecting {
                task.status = StreamStatus::Stopped;
                task.error = None;
            }
        }
        Ok(())
    }

    pub fn start_all(
        &self,
        app: &AppHandle,
        mediamtx: &MediaMtxState,
    ) -> Result<Vec<StreamTaskInfo>, String> {
        let ids: Vec<String> = self
            .tasks
            .lock()
            .unwrap()
            .iter()
            .filter(|(_, t)| {
                t.status != StreamStatus::Running && t.status != StreamStatus::Reconnecting
            })
            .map(|(id, _)| id.clone())
            .collect();

        let mut errors = Vec::new();
        for id in ids {
            if let Err(err) = self.start_stream(app, mediamtx, &id) {
                errors.push(format!("{id}: {err}"));
            }
        }

        if !errors.is_empty() && self.processes.lock().unwrap().is_empty() {
            return Err(errors.join("\n"));
        }

        Ok(self.list_streams())
    }

    pub fn stop_all(&self) -> Vec<StreamTaskInfo> {
        let ids: Vec<String> = self.processes.lock().unwrap().keys().cloned().collect();
        for id in ids {
            let _ = self.stop_stream_internal(&id);
        }
        self.list_streams()
    }

    pub fn stop_all_processes(&self) {
        let ids: Vec<String> = self.processes.lock().unwrap().keys().cloned().collect();
        for id in ids {
            let _ = self.stop_stream_internal(&id);
        }
    }

    pub fn system_info(&self, mediamtx: &MediaMtxState, app: &AppHandle) -> SystemInfo {
        let settings = self.get_settings();
        SystemInfo {
            lan_ip: self.lan_ip(),
            lan_ips: self.list_lan_ips(),
            mediamtx_running: mediamtx.is_running(),
            dependencies: mediamtx::dependency_status(app),
            settings,
            record_dir: mediamtx::record_dir(app, &self.get_settings())
                .to_string_lossy()
                .to_string(),
        }
    }
}

impl InternalStreamTask {
    fn clone_snapshot(&self) -> InternalStreamTask {
        InternalStreamTask {
            id: self.id.clone(),
            stream_name: self.stream_name.clone(),
            path: self.path.clone(),
            filename: self.filename.clone(),
            source_type: self.source_type.clone(),
            status: self.status.clone(),
            loop_enabled: self.loop_enabled,
            copy_mode: self.copy_mode,
            record_enabled: self.record_enabled,
            auto_reconnect: self.auto_reconnect,
            quality_preset: self.quality_preset.clone(),
            audio_device: self.audio_device.clone(),
            window_title: self.window_title.clone(),
            region: self.region.clone(),
            rtmp_url: self.rtmp_url.clone(),
            error: self.error.clone(),
            elapsed_secs: self.elapsed_secs,
            duration_secs: self.duration_secs,
            file_size_bytes: self.file_size_bytes,
            bitrate_kbps: self.bitrate_kbps,
            fps: self.fps,
            using_transcode: self.using_transcode,
            reconnect_attempts: self.reconnect_attempts,
            user_stopped: self.user_stopped,
        }
    }
}

async fn tokio_sleep(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
