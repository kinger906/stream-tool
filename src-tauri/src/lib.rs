mod capture;
mod commands;
mod ffmpeg;
mod mediamtx;
mod models;
mod network;
mod stream_manager;

use mediamtx::MediaMtxState;
use stream_manager::StreamManager;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .manage(StreamManager::new())
        .manage(MediaMtxState::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::list_lan_ips,
            commands::list_streams,
            commands::add_streams,
            commands::list_capture_devices,
            commands::list_windows,
            commands::add_camera_stream,
            commands::add_display_stream,
            commands::add_window_stream,
            commands::start_stream,
            commands::stop_stream,
            commands::start_all_streams,
            commands::stop_all_streams,
            commands::remove_stream,
            commands::update_stream,
            commands::update_settings,
            commands::start_mediamtx,
        ])
        .setup(|app| {
            let mediamtx = app.state::<MediaMtxState>();
            let manager = app.state::<StreamManager>();
            if let Err(err) = manager.init_mediamtx_config(app.handle(), &mediamtx) {
                eprintln!("MediaMTX 配置初始化失败: {err}");
            }
            if let Err(err) = mediamtx::start(app.handle(), &mediamtx) {
                eprintln!("MediaMTX 启动失败: {err}");
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let app = window.app_handle();
                if let Some(manager) = app.try_state::<StreamManager>() {
                    manager.stop_all_processes();
                }
                if let Some(mediamtx) = app.try_state::<MediaMtxState>() {
                    mediamtx::stop(&mediamtx);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
