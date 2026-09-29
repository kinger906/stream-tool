use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::LazyLock;

pub const RTSP_PORT: u16 = 8554;
pub const HLS_PORT: u16 = 8888;
pub const WEBRTC_PORT: u16 = 8889;

static STREAM_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]{2,32}$").unwrap());

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
    Window,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QualityPreset {
    Low,
    Medium,
    High,
}

impl Default for QualityPreset {
    fn default() -> Self {
        Self::Medium
    }
}

impl QualityPreset {
    pub fn video_size(&self) -> Option<&'static str> {
        match self {
            Self::Low => Some("640x360"),
            Self::Medium => Some("1280x720"),
            Self::High => Some("1920x1080"),
        }
    }

    pub fn video_bitrate(&self) -> &'static str {
        match self {
            Self::Low => "800k",
            Self::Medium => "2500k",
            Self::High => "4500k",
        }
    }

    pub fn framerate(&self) -> &'static str {
        match self {
            Self::Low => "24",
            Self::Medium => "30",
            Self::High => "30",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDevice {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureDevices {
    pub video: Vec<CaptureDevice>,
    pub audio: Vec<CaptureDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamTaskInfo {
    pub id: String,
    pub stream_name: String,
    pub filename: String,
    pub path: String,
    pub source_type: SourceType,
    pub status: StreamStatus,
    pub loop_enabled: bool,
    pub copy_mode: bool,
    pub record_enabled: bool,
    pub quality_preset: QualityPreset,
    pub audio_device: Option<String>,
    pub window_title: Option<String>,
    pub rtsp_url: String,
    pub hls_url: String,
    pub webrtc_url: String,
    pub rtsp_url_local: String,
    pub hls_url_local: String,
    pub webrtc_url_local: String,
    pub error: Option<String>,
    pub elapsed_secs: f64,
    pub duration_secs: Option<f64>,
    pub bitrate_kbps: Option<f64>,
    pub fps: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub max_concurrent: usize,
    pub selected_lan_ip: Option<String>,
    pub rtsp_username: Option<String>,
    pub rtsp_password: Option<String>,
    pub record_dir: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            selected_lan_ip: None,
            rtsp_username: None,
            rtsp_password: None,
            record_dir: None,
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
    pub lan_ips: Vec<String>,
    pub mediamtx_running: bool,
    pub dependencies: DependencyStatus,
    pub settings: AppSettings,
    pub record_dir: String,
}

#[derive(Clone)]
pub struct InternalStreamTask {
    pub id: String,
    pub stream_name: String,
    pub path: PathBuf,
    pub filename: String,
    pub source_type: SourceType,
    pub status: StreamStatus,
    pub loop_enabled: bool,
    pub copy_mode: bool,
    pub record_enabled: bool,
    pub quality_preset: QualityPreset,
    pub audio_device: Option<String>,
    pub window_title: Option<String>,
    pub error: Option<String>,
    pub elapsed_secs: f64,
    pub duration_secs: Option<f64>,
    pub bitrate_kbps: Option<f64>,
    pub fps: Option<f64>,
    pub using_transcode: bool,
}

pub fn validate_stream_name(name: &str) -> Result<(), String> {
    if STREAM_NAME_RE.is_match(name) {
        Ok(())
    } else {
        Err("流名称仅允许 2-32 位字母、数字、下划线或连字符".to_string())
    }
}

pub fn slug_from_filename(name: &str) -> String {
    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = slug.trim_matches('_');
    if trimmed.len() >= 2 {
        trimmed.chars().take(32).collect()
    } else {
        "stream".to_string()
    }
}

impl InternalStreamTask {
    pub fn to_info(&self, lan_ip: &str, settings: &AppSettings) -> StreamTaskInfo {
        let name = &self.stream_name;
        let auth = (
            settings.rtsp_username.as_deref(),
            settings.rtsp_password.as_deref(),
        );

        StreamTaskInfo {
            id: self.id.clone(),
            stream_name: name.clone(),
            filename: self.filename.clone(),
            path: self.path.display().to_string(),
            source_type: self.source_type.clone(),
            status: self.status.clone(),
            loop_enabled: self.loop_enabled,
            copy_mode: self.copy_mode,
            record_enabled: self.record_enabled,
            quality_preset: self.quality_preset.clone(),
            audio_device: self.audio_device.clone(),
            window_title: self.window_title.clone(),
            rtsp_url: build_rtsp_url(lan_ip, name, auth.0, auth.1),
            hls_url: build_hls_url(lan_ip, name, auth.0, auth.1),
            webrtc_url: build_webrtc_url(lan_ip, name),
            rtsp_url_local: build_rtsp_url("127.0.0.1", name, auth.0, auth.1),
            hls_url_local: build_hls_url("127.0.0.1", name, auth.0, auth.1),
            webrtc_url_local: build_webrtc_url("127.0.0.1", name),
            error: self.error.clone(),
            elapsed_secs: self.elapsed_secs,
            duration_secs: self.duration_secs,
            bitrate_kbps: self.bitrate_kbps,
            fps: self.fps,
        }
    }
}

fn build_rtsp_url(host: &str, name: &str, user: Option<&str>, pass: Option<&str>) -> String {
    match (user.filter(|s| !s.is_empty()), pass.filter(|s| !s.is_empty())) {
        (Some(u), Some(p)) => format!("rtsp://{u}:{p}@{host}:{RTSP_PORT}/{name}"),
        _ => format!("rtsp://{host}:{RTSP_PORT}/{name}"),
    }
}

fn build_hls_url(host: &str, name: &str, user: Option<&str>, pass: Option<&str>) -> String {
    match (user.filter(|s| !s.is_empty()), pass.filter(|s| !s.is_empty())) {
        (Some(u), Some(p)) => format!("http://{u}:{p}@{host}:{HLS_PORT}/{name}/index.m3u8"),
        _ => format!("http://{host}:{HLS_PORT}/{name}/index.m3u8"),
    }
}

fn build_webrtc_url(host: &str, name: &str) -> String {
    format!("http://{host}:{WEBRTC_PORT}/{name}/")
}
