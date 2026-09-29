use crate::ffmpeg::{
    build_probe_args, build_stream_args, parse_duration_from_stderr, parse_elapsed_from_stderr,
    parse_stats_from_stderr, summarize_error,
};
use crate::mediamtx::{self, MediaMtxState};
use crate::models::{
    slug_from_filename, validate_stream_name, AppSettings, InternalStreamTask, QualityPreset,
    SourceType, StreamStatus, StreamTaskInfo, SystemInfo,
};
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
        self.sync_mediamtx(app, mediamtx)?;
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
            quality_preset: QualityPreset::default(),
            audio_device,
            window_title,
            error: None,
            elapsed_secs: 0.0,
            duration_secs: None,
            bitrate_kbps: None,
            fps: None,
            using_transcode,
        }
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
        self.sync_mediamtx(app, mediamtx)?;
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

        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(id).ok_or_else(|| "任务不存在".to_string())?;

        if task.status == StreamStatus::Running {
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

        let settings = self.settings.lock().unwrap().clone();
        let lan_ip = self.lan_ip();
        let info = task.to_info(&lan_ip, &settings);
        drop(tasks);

        self.sync_mediamtx(app, mediamtx)?;
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
        }
        self.sync_mediamtx(app, mediamtx)?;
        Ok(self.get_settings())
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

        self.sync_mediamtx(app, mediamtx)?;

        if !mediamtx.is_running() {
            mediamtx::start(app, mediamtx)?;
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
            if task.source_type == SourceType::File {
                task.using_transcode = !task.copy_mode;
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

        let should_retry_transcode = {
            let tasks = manager.tasks.lock().unwrap();
            tasks
                .get(id)
                .map(|t| t.source_type == SourceType::File && t.copy_mode && !t.using_transcode)
                .unwrap_or(false)
        };

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
        if let Some(process) = self.processes.lock().unwrap().remove(id) {
            let _ = process.child.kill();
        }

        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(id) {
            if task.status == StreamStatus::Running {
                task.status = StreamStatus::Stopped;
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
            .filter(|(_, t)| t.status != StreamStatus::Running)
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
            quality_preset: self.quality_preset.clone(),
            audio_device: self.audio_device.clone(),
            window_title: self.window_title.clone(),
            error: self.error.clone(),
            elapsed_secs: self.elapsed_secs,
            duration_secs: self.duration_secs,
            bitrate_kbps: self.bitrate_kbps,
            fps: self.fps,
            using_transcode: self.using_transcode,
        }
    }
}

async fn tokio_sleep(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
