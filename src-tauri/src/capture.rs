use crate::models::{CaptureDevice, CaptureDevices};
use regex::Regex;
use std::sync::LazyLock;
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

// FFmpeg 7.x outputs [in#0 @ 0x...] instead of [dshow @ 0x...]
static VIDEO_DEVICE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\[[^\]]+\]\s+"([^"]+)"\s+\(video\)"#).unwrap());
static AUDIO_DEVICE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\[[^\]]+\]\s+"([^"]+)"\s+\(audio\)"#).unwrap());

pub async fn list_devices(app: &AppHandle) -> Result<CaptureDevices, String> {
    let sidecar = app.shell().sidecar("ffmpeg").map_err(|e| e.to_string())?;
    let (mut rx, _child) = sidecar
        .args([
            "-hide_banner".to_string(),
            "-list_devices".to_string(),
            "true".to_string(),
            "-f".to_string(),
            "dshow".to_string(),
            "-i".to_string(),
            "dummy".to_string(),
        ])
        .spawn()
        .map_err(|e| e.to_string())?;

    let mut output = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stderr(line) | CommandEvent::Stdout(line) => {
                output.push_str(&String::from_utf8_lossy(&line));
            }
            CommandEvent::Terminated(_) => break,
            _ => {}
        }
    }

    Ok(parse_dshow_devices(&output))
}

fn parse_dshow_devices(output: &str) -> CaptureDevices {
    let mut video = Vec::new();
    let mut audio = Vec::new();

    for cap in VIDEO_DEVICE_RE.captures_iter(output) {
        push_unique(&mut video, cap[1].to_string());
    }
    for cap in AUDIO_DEVICE_RE.captures_iter(output) {
        push_unique(&mut audio, cap[1].to_string());
    }

    CaptureDevices { video, audio }
}

fn push_unique(list: &mut Vec<CaptureDevice>, name: String) {
    if !list.iter().any(|d| d.name == name) {
        list.push(CaptureDevice { name });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ffmpeg7_device_list() {
        let sample = r#"
[in#0 @ 0000017c9e0c0b40] "Integrated Camera" (video)
[in#0 @ 0000017c9e0c0b40]   Alternative name "@device_pnp_..."
[in#0 @ 0000017c9e0c0b40] "WebcastMate VirtualCamera" (none)
[in#0 @ 0000017c9e0c0b40] "Microphone Array" (audio)
"#;
        let devices = parse_dshow_devices(sample);
        assert_eq!(devices.video.len(), 1);
        assert_eq!(devices.video[0].name, "Integrated Camera");
        assert_eq!(devices.audio.len(), 1);
        assert_eq!(devices.audio[0].name, "Microphone Array");
    }

    #[test]
    fn parse_legacy_dshow_format() {
        let sample = r#"
[dshow @ 000001abc] "USB Camera" (video)
[dshow @ 000001abc] "Stereo Mix" (audio)
"#;
        let devices = parse_dshow_devices(sample);
        assert_eq!(devices.video[0].name, "USB Camera");
        assert_eq!(devices.audio[0].name, "Stereo Mix");
    }
}
