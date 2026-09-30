use crate::models::{InternalStreamTask, QualityPreset, RTSP_PORT, SourceType};
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static DURATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Duration:\s(\d{2}):(\d{2}):(\d{2}\.\d{2})").unwrap());
static TIME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"time=(\d{2}):(\d{2}):(\d{2}\.\d{2})").unwrap());
static STATS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"fps=\s*([\d.]+).*bitrate=\s*([\d.]+)kbits/s").unwrap());

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

pub fn parse_stats_from_stderr(stderr: &str) -> (Option<f64>, Option<f64>) {
    STATS_RE
        .captures_iter(stderr)
        .last()
        .map(|caps| (caps[1].parse().ok(), caps[2].parse().ok()))
        .unwrap_or((None, None))
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
                || lower.contains("connection refused")
                || lower.contains("server error")
        })
        .or_else(|| lines.last())
        .map(|line| line.trim().to_string())
        .unwrap_or_else(|| "FFmpeg 推流失败".to_string())
}

fn append_outputs(args: &mut Vec<String>, task: &InternalStreamTask) {
    args.extend([
        "-f".to_string(),
        "rtsp".to_string(),
        "-rtsp_transport".to_string(),
        "tcp".to_string(),
        format!("rtsp://127.0.0.1:{RTSP_PORT}/{}", task.stream_name),
    ]);

    if let Some(rtmp) = task
        .rtmp_url
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        args.extend(["-f".to_string(), "flv".to_string(), rtmp.to_string()]);
    }
}

fn transcode_video_args(preset: &QualityPreset) -> Vec<String> {
    let mut args = vec![
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-tune".to_string(),
        "zerolatency".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-b:v".to_string(),
        preset.video_bitrate().to_string(),
    ];
    if let Some(size) = preset.video_size() {
        args.push("-s".to_string());
        args.push(size.to_string());
    }
    args
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

pub fn build_stream_args(task: &InternalStreamTask, transcode: bool) -> Vec<String> {
    match task.source_type {
        SourceType::File => build_file_stream_args(task, transcode),
        SourceType::Camera => build_camera_stream_args(task),
        SourceType::Display => build_display_stream_args(task, "desktop", None),
        SourceType::Window => build_display_stream_args(
            task,
            &format!("title={}", task.window_title.clone().unwrap_or_default()),
            None,
        ),
        SourceType::Region => {
            let region = task.region.clone().unwrap_or(crate::models::CaptureRegion {
                x: 0,
                y: 0,
                width: 1280,
                height: 720,
            });
            build_display_stream_args(task, "desktop", Some(region))
        }
    }
}

fn append_large_file_open_flags(args: &mut Vec<String>) {
    // Cap probe so multi-GB files open quickly instead of scanning the whole container.
    args.extend([
        "-probesize".to_string(),
        "5M".to_string(),
        "-analyzeduration".to_string(),
        "10M".to_string(),
        "-fflags".to_string(),
        "+genpts".to_string(),
    ]);
}

fn build_file_stream_args(task: &InternalStreamTask, transcode: bool) -> Vec<String> {
    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "info".to_string(),
    ];

    if task.loop_enabled {
        args.push("-stream_loop".to_string());
        args.push("-1".to_string());
    }

    args.push("-re".to_string());
    append_large_file_open_flags(&mut args);
    args.push("-i".to_string());
    args.push(task.path.display().to_string());

    // RTMP (FLV) almost always needs re-encode; keep copy only for RTSP-only.
    let force_transcode = transcode
        || task
            .rtmp_url
            .as_ref()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);

    if force_transcode {
        args.extend(transcode_video_args(&task.quality_preset));
        args.extend(transcode_audio_args());
    } else {
        // Prefer copy for large files: terminals pull a live stream instead of
        // opening the multi-GB file and buffering the whole container.
        args.push("-c".to_string());
        args.push("copy".to_string());
    }

    append_outputs(&mut args, task);
    args
}

fn dshow_video_input(video: &str, audio: Option<&str>) -> String {
    match audio {
        Some(a) => format!("video={video}:audio={a}"),
        None => format!("video={video}"),
    }
}

fn dshow_audio_input(audio: &str) -> String {
    format!("audio={audio}")
}

fn build_camera_stream_args(task: &InternalStreamTask) -> Vec<String> {
    let video = task.path.to_string_lossy();
    let preset = &task.quality_preset;
    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "info".to_string(),
        "-f".to_string(),
        "dshow".to_string(),
        "-rtbufsize".to_string(),
        "100M".to_string(),
        "-i".to_string(),
        dshow_video_input(&video, task.audio_device.as_deref()),
    ];

    args.extend(transcode_video_args(preset));
    args.push("-r".to_string());
    args.push(preset.framerate().to_string());
    if task.audio_device.is_some() {
        args.extend(transcode_audio_args());
    } else {
        args.push("-an".to_string());
    }

    append_outputs(&mut args, task);
    args
}

fn build_display_stream_args(
    task: &InternalStreamTask,
    target: &str,
    region: Option<crate::models::CaptureRegion>,
) -> Vec<String> {
    let preset = &task.quality_preset;
    let mut args = vec![
        "-hide_banner".to_string(),
        "-loglevel".to_string(),
        "info".to_string(),
        "-f".to_string(),
        "gdigrab".to_string(),
        "-framerate".to_string(),
        preset.framerate().to_string(),
        "-draw_mouse".to_string(),
        "1".to_string(),
    ];

    if let Some(r) = region {
        args.extend([
            "-offset_x".to_string(),
            r.x.to_string(),
            "-offset_y".to_string(),
            r.y.to_string(),
            "-video_size".to_string(),
            format!("{}x{}", r.width, r.height),
        ]);
    }

    args.push("-i".to_string());
    args.push(target.to_string());

    if let Some(audio) = &task.audio_device {
        args.extend([
            "-f".to_string(),
            "dshow".to_string(),
            "-i".to_string(),
            dshow_audio_input(audio),
        ]);
    }

    args.extend(transcode_video_args(preset));
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

    append_outputs(&mut args, task);
    args
}

pub fn build_probe_args(file_path: &Path) -> Vec<String> {
    vec![
        "-hide_banner".to_string(),
        "-probesize".to_string(),
        "5M".to_string(),
        "-analyzeduration".to_string(),
        "10M".to_string(),
        "-i".to_string(),
        file_path.display().to_string(),
    ]
}

pub fn sidecar_target_hint(name: &str) -> String {
    format!("src-tauri/binaries/{name}-x86_64-pc-windows-msvc.exe")
}
