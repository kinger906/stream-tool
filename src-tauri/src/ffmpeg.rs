use crate::models::{InternalStreamTask, RTSP_PORT, SourceType};
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
                || lower.contains("cannot find")
        })
        .or_else(|| lines.last())
        .map(|line| line.trim().to_string())
        .unwrap_or_else(|| "FFmpeg 推流失败".to_string())
}

fn rtsp_output(stream_id: &str) -> Vec<String> {
    vec![
        "-f".to_string(),
        "rtsp".to_string(),
        "-rtsp_transport".to_string(),
        "tcp".to_string(),
        format!("rtsp://127.0.0.1:{RTSP_PORT}/{stream_id}"),
    ]
}

fn transcode_video_args() -> Vec<String> {
    vec![
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-tune".to_string(),
        "zerolatency".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
    ]
}

fn transcode_audio_args() -> Vec<String> {
    vec![
        "-c:a".to_string(),
        "aac".to_string(),
        "-ar".to_string(),
        "44100".to_string(),
        "-ac".to_string(),
        "2".to_string(),
    ]
}

pub fn build_stream_args(task: &InternalStreamTask, stream_id: &str, transcode: bool) -> Vec<String> {
    match task.source_type {
        SourceType::File => build_file_stream_args(task, stream_id, transcode),
        SourceType::Camera => build_camera_stream_args(task, stream_id),
        SourceType::Display => build_display_stream_args(task, stream_id),
    }
}

fn build_file_stream_args(task: &InternalStreamTask, stream_id: &str, transcode: bool) -> Vec<String> {
    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "warning".to_string(),
    ];

    if task.loop_enabled {
        args.push("-stream_loop".to_string());
        args.push("-1".to_string());
    }

    args.push("-re".to_string());
    args.push("-i".to_string());
    args.push(task.path.display().to_string());

    if transcode {
        args.extend(transcode_video_args());
        args.extend(transcode_audio_args());
    } else {
        args.push("-c".to_string());
        args.push("copy".to_string());
    }

    args.extend(rtsp_output(stream_id));
    args
}

fn build_camera_stream_args(task: &InternalStreamTask, stream_id: &str) -> Vec<String> {
    let video = task.path.to_string_lossy();
    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "warning".to_string(),
        "-f".to_string(),
        "dshow".to_string(),
        "-video_size".to_string(),
        "1280x720".to_string(),
        "-framerate".to_string(),
        "30".to_string(),
    ];

    let input = match &task.audio_device {
        Some(audio) => format!("video={video}:audio={audio}"),
        None => format!("video={video}"),
    };
    args.push("-i".to_string());
    args.push(input);

    args.extend(transcode_video_args());
    if task.audio_device.is_some() {
        args.extend(transcode_audio_args());
    } else {
        args.push("-an".to_string());
    }

    args.extend(rtsp_output(stream_id));
    args
}

fn build_display_stream_args(task: &InternalStreamTask, stream_id: &str) -> Vec<String> {
    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "warning".to_string(),
        "-f".to_string(),
        "gdigrab".to_string(),
        "-framerate".to_string(),
        "25".to_string(),
        "-draw_mouse".to_string(),
        "1".to_string(),
        "-i".to_string(),
        "desktop".to_string(),
    ];

    if let Some(audio) = &task.audio_device {
        args.extend([
            "-f".to_string(),
            "dshow".to_string(),
            "-i".to_string(),
            format!("audio={audio}"),
        ]);
    }

    args.extend(transcode_video_args());
    if task.audio_device.is_some() {
        args.extend(transcode_audio_args());
        args.extend([
            "-map".to_string(),
            "0:v".to_string(),
            "-map".to_string(),
            "1:a".to_string(),
        ]);
    } else {
        args.push("-an".to_string());
    }

    args.extend(rtsp_output(stream_id));
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
