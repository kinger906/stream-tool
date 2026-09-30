mod capture;
mod commands;
mod ffmpeg;
mod mediamtx;
mod models;
mod network;
mod scenes;
mod stream_manager;

use mediamtx::MediaMtxState;
use stream_manager::StreamManager;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

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
            commands::add_region_stream,
            commands::start_stream,
            commands::stop_stream,
            commands::start_all_streams,
            commands::stop_all_streams,
            commands::remove_stream,
            commands::update_stream,
            commands::update_settings,
            commands::list_scenes,
            commands::save_scene,
            commands::delete_scene,
            commands::apply_scene,
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

            if let Err(err) = setup_tray(app.handle()) {
                eprintln!("系统托盘初始化失败: {err}");
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let minimize = app
                    .try_state::<StreamManager>()
                    .map(|m| m.get_settings().minimize_to_tray)
                    .unwrap_or(true);
                if minimize {
                    api.prevent_close();
                    let _ = window.hide();
                    return;
                }

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

fn setup_tray(app: &tauri::AppHandle) -> Result<(), String> {
    let show_i = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let start_i = MenuItem::with_id(app, "start_all", "全部开始", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let stop_i = MenuItem::with_id(app, "stop_all", "全部停止", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit_i =
        MenuItem::with_id(app, "quit", "退出", true, None::<&str>).map_err(|e| e.to_string())?;
    let menu = Menu::with_items(app, &[&show_i, &start_i, &stop_i, &quit_i])
        .map_err(|e| e.to_string())?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().ok_or("缺少应用图标")?)
        .menu(&menu)
        .tooltip("视频推流工具")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "start_all" => {
                if let (Some(manager), Some(mediamtx)) = (
                    app.try_state::<StreamManager>(),
                    app.try_state::<MediaMtxState>(),
                ) {
                    let _ = manager.start_all(app, &mediamtx);
                }
            }
            "stop_all" => {
                if let Some(manager) = app.try_state::<StreamManager>() {
                    manager.stop_all();
                }
            }
            "quit" => {
                if let Some(manager) = app.try_state::<StreamManager>() {
                    manager.stop_all_processes();
                }
                if let Some(mediamtx) = app.try_state::<MediaMtxState>() {
                    mediamtx::stop(&mediamtx);
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(())
}
