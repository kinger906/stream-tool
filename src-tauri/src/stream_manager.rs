use crate::ffmpeg::{
    build_probe_args, build_stream_args, parse_duration_from_stderr, parse_elapsed_from_stderr,
    summarize_error,
};
use crate::mediamtx::MediaMtxState;
use crate::models::{
    AppSettings, InternalStreamTask, SourceType, StreamStatus, StreamTaskInfo, SystemInfo,
};
use local_ip_address::local_ip;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
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

    pub fn set_max_concurrent(&self, max: usize) -> AppSettings {
        let mut settings = self.settings.lock().unwrap();
        settings.max_concurrent = max.clamp(1, 32);
        settings.clone()
    }

    pub fn lan_ip(&self) -> String {
        local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "127.0.0.1".to_string())
    }

    pub fn list_streams(&self) -> Vec<StreamTaskInfo> {
        let lan_ip = self.lan_ip();
        self.tasks
            .lock()
            .unwrap()
            .values()
            .map(|task| task.to_info(&lan_ip))
            .collect()
    }

    pub fn add_files(&self, paths: Vec<String>) -> Result<Vec<StreamTaskInfo>, String> {
        let mut added = Vec::new();
        let lan_ip = self.lan_ip();

        for path_str in paths {
            let path = PathBuf::from(&path_str);
            if !path.exists() {
                continue;
            }

            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path_str.clone());

            let id = Uuid::new_v4().simple().to_string()[..8].to_string();

            let task = InternalStreamTask {
                id: id.clone(),
                path: path.clone(),
                filename,
                source_type: SourceType::File,
                status: StreamStatus::Idle,
                loop_enabled: true,
                copy_mode: true,
                error: None,
                elapsed_secs: 0.0,
                duration_secs: None,
                using_transcode: false,
            };

            self.tasks.lock().unwrap().insert(id.clone(), task);
            if let Some(info) = self.tasks.lock().unwrap().get(&id).map(|t| t.to_info(&lan_ip)) {
                added.push(info);
            }
        }

        Ok(added)
    }

    pub fn remove_stream(&self, id: &str) -> Result<(), String> {
        self.stop_stream_internal(id)?;
        self.tasks.lock().unwrap().remove(id);
        Ok(())
    }

    pub fn update_stream(
        &self,
        id: &str,
        loop_enabled: Option<bool>,
        copy_mode: Option<bool>,
    ) -> Result<StreamTaskInfo, String> {
        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks.get_mut(id).ok_or_else(|| "任务不存在".to_string())?;

        if task.status == StreamStatus::Running {
            return Err("推流进行中，无法修改设置".to_string());
        }

        if let Some(v) = loop_enabled {
            task.loop_enabled = v;
        }
        if let Some(v) = copy_mode {
            task.copy_mode = v;
        }

        Ok(task.to_info(&self.lan_ip()))
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

        if !mediamtx.is_running() {
            crate::mediamtx::start(app, mediamtx)?;
        }

        let running_count = self
            .processes
            .lock()
            .unwrap()
            .len();
        let max = self.settings.lock().unwrap().max_concurrent;
        if running_count >= max {
            return Err(format!("已达并发上限 ({max})，请先停止其它推流或提高上限"));
        }

        if self.processes.lock().unwrap().contains_key(id) {
            return Err("该任务已在推流中".to_string());
        }

        let (path, loop_enabled, copy_mode) = {
            let mut tasks = self.tasks.lock().unwrap();
            let task = tasks.get_mut(id).ok_or_else(|| "任务不存在".to_string())?;
            if task.source_type != SourceType::File {
                return Err("该来源类型尚未支持".to_string());
            }
            task.status = StreamStatus::Running;
            task.error = None;
            task.elapsed_secs = 0.0;
            task.using_transcode = !task.copy_mode;
            (
                task.path.clone(),
                task.loop_enabled,
                !task.copy_mode,
            )
        };

        let app_clone = app.clone();
        let id_owned = id.to_string();

        match self.spawn_ffmpeg(
            app.clone(),
            &id_owned,
            &path,
            loop_enabled,
            copy_mode,
            move |stream_id, stderr, exit_code| {
                Self::handle_process_exit(&app_clone, &stream_id, &stderr, exit_code);
            },
        ) {
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

        self.probe_duration_async(app.clone(), id.to_string(), path);

        Ok(self
            .tasks
            .lock()
            .unwrap()
            .get(id)
            .map(|t| t.to_info(&self.lan_ip()))
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
                .map(|t| t.copy_mode && !t.using_transcode)
                .unwrap_or(false)
        };

        if exit_code.is_none() || exit_code == Some(0) {
            let mut tasks = manager.tasks.lock().unwrap();
            if let Some(task) = tasks.get_mut(id) {
                if task.loop_enabled {
                    task.status = StreamStatus::Running;
                } else {
                    task.status = StreamStatus::Stopped;
                }
            }
            let _ = app.emit("streams-changed", ());
            return;
        }

        if should_retry_transcode {
            let (path, loop_enabled) = {
                let mut tasks = manager.tasks.lock().unwrap();
                let task = tasks.get_mut(id).unwrap();
                task.using_transcode = true;
                task.status = StreamStatus::Running;
                task.error = Some("Copy 模式失败，正在尝试转码推流…".to_string());
                (task.path.clone(), task.loop_enabled)
            };

            let app_clone = app.clone();
            let id_owned = id.to_string();

            if manager
                .spawn_ffmpeg(
                    app.clone(),
                    &id_owned,
                    &path,
                    loop_enabled,
                    true,
                    move |stream_id, stderr, exit_code| {
                        Self::handle_process_exit(&app_clone, &stream_id, &stderr, exit_code);
                    },
                )
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
        path: &Path,
        loop_enabled: bool,
        transcode: bool,
        on_exit: F,
    ) -> Result<(), String>
    where
        F: FnOnce(String, String, Option<i32>) + Send + 'static,
    {
        let args = build_stream_args(path, id, loop_enabled, transcode);
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

                let elapsed = {
                    let processes = manager.processes.lock().unwrap();
                    processes
                        .get(&id_for_poll)
                        .and_then(|p| parse_elapsed_from_stderr(&p.stderr.lock().unwrap()))
                };

                if let Some(elapsed) = elapsed {
                    let mut tasks = manager.tasks.lock().unwrap();
                    if let Some(task) = tasks.get_mut(&id_for_poll) {
                        task.elapsed_secs = elapsed;
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
        let lan_ip = self.lan_ip();
        Ok(self
            .tasks
            .lock()
            .unwrap()
            .get(id)
            .map(|t| t.to_info(&lan_ip))
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
            .filter(|(_, t)| t.status != StreamStatus::Running && t.source_type == SourceType::File)
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
        SystemInfo {
            lan_ip: self.lan_ip(),
            mediamtx_running: mediamtx.is_running(),
            dependencies: crate::mediamtx::dependency_status(app),
            settings: self.get_settings(),
        }
    }
}

async fn tokio_sleep(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}
