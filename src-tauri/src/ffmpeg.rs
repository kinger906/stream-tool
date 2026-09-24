use crate::models::RTSP_PORT;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static DURATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Duration:\s(\d{2}):(\d{2}):(\d{2}\.\d{2})").unwrap());
static TIME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"time=(\d{2}):(\d{2}):(\d{2}\.\d{2})").unwrap());

pub fn parse_hms_to_secs(h: &str, m: &str, s: &str) -> f64 {
    h.parse::<f64>().unwrap_or(0.0) * 3600.0
        + m.parse::<f64>().unwrap_or(0.0) * 60.0
        + s.parse::<f64>().unwrap_or(0.0)
}

pub fn parse_duration_from_stderr(stderr: &str) -> Option<f64> {
    DURATION_RE
        .captures(stderr)
        .map(|caps| parse_hms_to_secs(&caps[1], &caps[2], &caps[3]))
}

pub fn parse_elapsed_from_stderr(stderr: &str) -> Option<f64> {
    TIME_RE
        .captures_iter(stderr)
        .last()
        .map(|caps| parse_hms_to_secs(&caps[1], &caps[2], &caps[3]))
}

pub fn summarize_error(stderr: &str) -> String {
    let lines: Vec<&str> = stderr
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            !lower.starts_with("ffmpeg version")
                && !lower.starts_with("  configuration:")
                && !lower.starts_with("  libavutil")
                && !lower.starts_with("  libavcodec")
                && !lower.starts_with("  libavformat")
                && !lower.starts_with("  libswscale")
                && !lower.starts_with("  libswresample")
                && !lower.starts_with("  libpostproc")
                && !line.trim().is_empty()
        })
        .collect();

    lines
        .iter()
        .rev()
        .find(|line| {
            let lower = line.to_lowercase();
            lower.contains("error")
                || lower.contains("invalid")
                || lower.contains("failed")
                || lower.contains("no such file")
        })
        .or_else(|| lines.last())
        .map(|line| line.trim().to_string())
        .unwrap_or_else(|| "FFmpeg 推流失败".to_string())
}

pub fn build_stream_args(
    file_path: &Path,
    stream_id: &str,
    loop_enabled: bool,
    transcode: bool,
) -> Vec<String> {
    let mut args = vec!["-hide_banner".to_string(), "-loglevel".to_string(), "warning".to_string()];

    if loop_enabled {
        args.push("-stream_loop".to_string());
        args.push("-1".to_string());
    }

    args.push("-re".to_string());
    args.push("-i".to_string());
    args.push(file_path.display().to_string());

    if transcode {
        args.extend([
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryfast".to_string(),
            "-tune".to_string(),
            "zerolatency".to_string(),
            "-c:a".to_string(),
            "aac".to_string(),
        ]);
    } else {
        args.push("-c".to_string());
        args.push("copy".to_string());
    }

    args.extend([
        "-f".to_string(),
        "rtsp".to_string(),
        "-rtsp_transport".to_string(),
        "tcp".to_string(),
        format!("rtsp://127.0.0.1:{RTSP_PORT}/{stream_id}"),
    ]);

    args
}

pub fn build_probe_args(file_path: &Path) -> Vec<String> {
    vec![
        "-hide_banner".to_string(),
        "-i".to_string(),
        file_path.display().to_string(),
    ]
}

pub fn sidecar_target_hint(name: &str) -> String {
    format!("src-tauri/binaries/{name}-x86_64-pc-windows-msvc.exe")
}
