#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod auth;
mod commands;
mod login;
mod lyrics;
mod parse;
mod state;
mod store;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use commands::AppState;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, WindowEvent};

const MAIN: &str = "main";

fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// 托盘菜单里的播放控制交给前端执行：音频元素在 webview 里，Rust 侧碰不到。
fn emit_player(app: &tauri::AppHandle, action: &str) {
    let _ = app.emit("tray-player", action);
}

fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let prev = MenuItem::with_id(app, "prev", "上一首", true, None::<&str>)?;
    let toggle = MenuItem::with_id(app, "toggle", "播放 / 暂停", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", "下一首", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &show,
            &PredefinedMenuItem::separator(app)?,
            &prev,
            &toggle,
            &next,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("音乐")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "prev" => emit_player(app, "previous"),
            "toggle" => emit_player(app, "toggle"),
            "next" => emit_player(app, "next"),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });

    // 托盘用单独的高亮图标：应用图标是深色底，放在深色任务栏上几乎看不见。
    match tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png")) {
        Ok(icon) => builder = builder.icon(icon),
        Err(_) => {
            if let Some(icon) = app.default_window_icon() {
                builder = builder.icon(icon.clone());
            }
        }
    }
    builder.build(app)?;
    Ok(())
}

fn main() {
    // 窗口尺寸与位置由官方插件接管，退出时自动落盘、下次启动自动恢复。
    // 但必须排除 DECORATIONS：否则它会用上次保存的 decorated=true 覆盖配置里的无边框设置。
    let state_flags = tauri_plugin_window_state::StateFlags::all()
        & !tauri_plugin_window_state::StateFlags::DECORATIONS;

    tauri::Builder::default()
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(state_flags)
                .build(),
        )
        .setup(|app| {
            let snapshot = state::load_state(&app.handle().clone());
            let restored = store::load_session();
            if restored.is_some() {
                eprintln!("[session] 已从系统密钥环恢复音乐会话");
            }
            app.manage(AppState {
                client: api::Client::new(),
                session: Mutex::new(restored),
                close_to_tray: AtomicBool::new(snapshot.close_to_tray),
            });

            if let Err(error) = build_tray(&app.handle().clone()) {
                eprintln!("[tray] 托盘创建失败：{error}");
            }

            // 关闭窗口默认隐藏到托盘；关掉这个选项后恢复成直接退出。
            if let Some(window) = app.get_webview_window(MAIN) {
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let state = handle.state::<AppState>();
                        if state.close_to_tray.load(Ordering::Relaxed) {
                            api.prevent_close();
                            if let Some(window) = handle.get_webview_window(MAIN) {
                                let _ = window.hide();
                            }
                        }
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth_status,
            commands::login,
            commands::logout,
            commands::set_close_to_tray,
            commands::favorites,
            commands::search,
            commands::playback,
            commands::favorite_exist,
            commands::set_favorite,
            commands::recent,
            commands::playlists,
            commands::playlist_detail,
            commands::album,
            commands::artist,
            commands::lyrics,
            commands::load_state,
            commands::save_state,
            commands::lyrics_cache_size,
            commands::clear_lyrics_cache,
        ])
        .run(tauri::generate_context!())
        .expect("应用启动失败");
}
