use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{
    menu::{MenuBuilder, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_opener::OpenerExt;

const BASE_URL: &str = "https://lich.ai.vn/";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DesktopSettings {
    notifications: bool,
    day_icon: bool,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            notifications: true,
            day_icon: true,
        }
    }
}

fn settings_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("desktop-settings.json")
}

fn load_settings<R: Runtime>(app: &AppHandle<R>) -> DesktopSettings {
    let path = settings_path(app);
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_settings<R: Runtime>(app: &AppHandle<R>, value: &DesktopSettings) -> Result<(), String> {
    let path = settings_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

fn route_url(route: &str) -> String {
    // Keep navigation web-first. Until dedicated deep links are finalized,
    // routes are sent as a harmless query hint and the web homepage remains canonical.
    if route.trim().is_empty() {
        BASE_URL.to_string()
    } else {
        format!("{}?desktop={}", BASE_URL, urlencoding::encode(route))
    }
}

fn open_route<R: Runtime>(app: &AppHandle<R>, route: &str) {
    let _ = app
        .opener()
        .open_url(route_url(route), None::<&str>);
}

fn show_main<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn open_lichai(app: AppHandle, route: String) {
    open_route(&app, &route);
}

#[tauri::command]
fn hide_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn autostart_status(app: AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    if enabled {
        app.autolaunch().enable().map_err(|e| e.to_string())
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn get_settings(app: AppHandle) -> DesktopSettings {
    load_settings(&app)
}

#[tauri::command]
fn set_setting(app: AppHandle, key: String, value: bool) -> Result<(), String> {
    let mut s = load_settings(&app);
    match key.as_str() {
        "notifications" => s.notifications = value,
        "day_icon" => s.day_icon = value,
        _ => return Err("Unknown setting".into()),
    }
    save_settings(&app, &s)
}

fn build_tray<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    // Account: all features remain visible even before login.
    let account = SubmenuBuilder::new(app, "👤 Tài khoản")
        .text("account_login", "Đăng nhập / Đăng ký")
        .text("account_manage", "Tài khoản & bảo mật")
        .build()?;

    let calendar = SubmenuBuilder::new(app, "📅 Lịch")
        .text("calendar_today", "Hôm nay")
        .text("calendar_month", "Lịch tháng")
        .text("calendar_lunar", "Âm lịch")
        .text("calendar_goodday", "Ngày tốt")
        .text("calendar_goodhour", "Giờ tốt")
        .build()?;

    let personal = SubmenuBuilder::new(app, "📝 Cá nhân")
        .text("personal_notes", "Ghi chú")
        .text("personal_events", "Sự kiện")
        .text("personal_birthdays", "Sinh nhật")
        .text("personal_anniversary", "Kỷ niệm / Ngày giỗ")
        .build()?;

    let utilities = SubmenuBuilder::new(app, "🧭 Tiện ích")
        .text("utils_weather", "Thời tiết")
        .text("utils_fengshui", "Ngày giờ & hướng tốt")
        .text("utils_career", "Học & Nghề")
        .text("utils_more", "Các tiện ích khác")
        .build()?;

    let menu = MenuBuilder::new(app)
        .text("open_home", "🌐 Mở LịchAI")
        .separator()
        .item(&account)
        .item(&calendar)
        .item(&personal)
        .item(&utilities)
        .separator()
        .text("settings", "⚙️ Cài đặt")
        .text("quit", "Thoát")
        .build()?;

    TrayIconBuilder::with_id("lichai-tray")
        .tooltip("LịchAI")
        .icon(app.default_window_icon().expect("missing app icon").clone())
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open_home" => open_route(app, ""),
            "account_login" => open_route(app, "login"),
            "account_manage" => open_route(app, "account"),
            "calendar_today" => open_route(app, "today"),
            "calendar_month" => open_route(app, "month"),
            "calendar_lunar" => open_route(app, "lunar"),
            "calendar_goodday" => open_route(app, "good-day"),
            "calendar_goodhour" => open_route(app, "good-hour"),
            "personal_notes" => open_route(app, "notes"),
            "personal_events" => open_route(app, "events"),
            "personal_birthdays" => open_route(app, "birthdays"),
            "personal_anniversary" => open_route(app, "anniversary"),
            "utils_weather" => open_route(app, "weather"),
            "utils_fengshui" => open_route(app, "fengshui"),
            "utils_career" => open_route(app, "career"),
            "utils_more" => open_route(app, "utilities"),
            "settings" => show_main(app),
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
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                open_route(tray.app_handle(), "");
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            open_lichai,
            hide_main_window,
            autostart_status,
            set_autostart,
            get_settings,
            set_setting
        ])
        .setup(|app| {
            build_tray(app)?;

            // Tray-first: do not show the small utility window at startup.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(w) = handle.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running LichAI Desktop");
}
