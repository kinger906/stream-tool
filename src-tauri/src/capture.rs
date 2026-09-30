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

pub fn list_windows() -> Result<Vec<CaptureDevice>, String> {
    #[cfg(windows)]
    {
        list_windows_win32()
    }

    #[cfg(not(windows))]
    {
        Err("窗口采集仅支持 Windows".to_string())
    }
}

/// Enumerate visible top-level window titles via Win32 UTF-16 APIs
/// (avoids PowerShell GBK/UTF-8 mojibake on Chinese Windows).
#[cfg(windows)]
fn list_windows_win32() -> Result<Vec<CaptureDevice>, String> {
    struct EnumState {
        titles: Vec<String>,
    }

    unsafe extern "system" fn enum_proc(hwnd: isize, lparam: isize) -> i32 {
        const GW_OWNER: u32 = 4;

        // SAFETY: lparam points to EnumState living on the calling stack for the
        // duration of EnumWindows.
        let state = unsafe { &mut *(lparam as *mut EnumState) };

        unsafe {
            if IsWindowVisible(hwnd) == 0 {
                return 1;
            }
            // Skip owned popups; keep top-level windows.
            if GetWindow(hwnd, GW_OWNER) != 0 {
                return 1;
            }

            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            if len <= 0 {
                return 1;
            }

            let title = String::from_utf16_lossy(&buf[..len as usize]);
            let title = title.trim();
            if title.is_empty() {
                return 1;
            }

            if !state.titles.iter().any(|t| t == title) {
                state.titles.push(title.to_string());
            }
        }
        1
    }

    let mut state = EnumState {
        titles: Vec::new(),
    };
    let ok = unsafe { EnumWindows(Some(enum_proc), &mut state as *mut EnumState as isize) };
    if ok == 0 {
        return Err("枚举窗口失败".to_string());
    }

    state.titles.sort();
    Ok(state
        .titles
        .into_iter()
        .map(|name| CaptureDevice { name })
        .collect())
}

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn EnumWindows(
        lp_enum_func: Option<unsafe extern "system" fn(isize, isize) -> i32>,
        lparam: isize,
    ) -> i32;
    fn IsWindowVisible(hwnd: isize) -> i32;
    fn GetWindow(hwnd: isize, cmd: u32) -> isize;
    fn GetWindowTextW(hwnd: isize, lp_string: *mut u16, n_max_count: i32) -> i32;
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
