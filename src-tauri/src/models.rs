use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const RTSP_PORT: u16 = 8554;
pub const HLS_PORT: u16 = 8888;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamStatus {
    Idle,
    Running,
    Stopped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    File,
    Camera,
    Display,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamTaskInfo {
    pub id: String,
    pub filename: String,
    pub path: String,
    pub source_type: SourceType,
    pub status: StreamStatus,
    pub loop_enabled: bool,
    pub copy_mode: bool,
    pub rtsp_url: String,
    pub hls_url: String,
    pub rtsp_url_local: String,
    pub hls_url_local: String,
    pub error: Option<String>,
    pub elapsed_secs: f64,
    pub duration_secs: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub max_concurrent: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyStatus {
    pub ffmpeg_available: bool,
    pub mediamtx_available: bool,
    pub ffmpeg_path_hint: String,
    pub mediamtx_path_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub lan_ip: String,
    pub mediamtx_running: bool,
    pub dependencies: DependencyStatus,
    pub settings: AppSettings,
}

pub struct InternalStreamTask {
    pub id: String,
    pub path: PathBuf,
    pub filename: String,
    pub source_type: SourceType,
    pub status: StreamStatus,
    pub loop_enabled: bool,
    pub copy_mode: bool,
    pub error: Option<String>,
    pub elapsed_secs: f64,
    pub duration_secs: Option<f64>,
    pub using_transcode: bool,
}

impl InternalStreamTask {
    pub fn to_info(&self, lan_ip: &str) -> StreamTaskInfo {
        let id = &self.id;
        StreamTaskInfo {
            id: id.clone(),
            filename: self.filename.clone(),
            path: self.path.display().to_string(),
            source_type: self.source_type.clone(),
            status: self.status.clone(),
            loop_enabled: self.loop_enabled,
            copy_mode: self.copy_mode,
            rtsp_url: format!("rtsp://{lan_ip}:{RTSP_PORT}/{id}"),
            hls_url: format!("http://{lan_ip}:{HLS_PORT}/{id}/index.m3u8"),
            rtsp_url_local: format!("rtsp://127.0.0.1:{RTSP_PORT}/{id}"),
            hls_url_local: format!("http://127.0.0.1:{HLS_PORT}/{id}/index.m3u8"),
            error: self.error.clone(),
            elapsed_secs: self.elapsed_secs,
            duration_secs: self.duration_secs,
        }
    }
}
