use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use tauri_plugin_autostart::ManagerExt as AutostartManagerExt;
use tauri_plugin_opener::OpenerExt;
use chrono::{Datelike, Local, NaiveDate};

const BASE_URL: &str = "https://lich.ai.vn/";
const ABOUT_URL: &str = "https://lich.ai.vn/?desktop=about";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DesktopSettings {
    notifications: bool,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self { notifications: true }
    }
}

#[derive(Debug, Clone, Serialize)]
struct DayInfo {
    day: u32,
    month: u32,
    year: i32,
    weekday: String,
    solar: String,
    lunar: String,
    lunar_year_can_chi: String,
    day_can_chi: String,
    good_hours: String,
    good_direction: String,
    event: Option<String>,
}

fn settings_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("desktop-settings.json")
}

fn load_settings<R: Runtime>(app: &AppHandle<R>) -> DesktopSettings {
    fs::read_to_string(settings_path(app))
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

fn open_home<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.opener().open_url(BASE_URL, None::<&str>);
}

fn open_about<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.opener().open_url(ABOUT_URL, None::<&str>);
}

fn show_mode<R: Runtime>(app: &AppHandle<R>, mode: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let js = format!(
            "window.setLichAIMode && window.setLichAIMode({:?});",
            mode
        );
        let _ = window.eval(&js);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn open_lichai(app: AppHandle) {
    open_home(&app);
}

#[tauri::command]
fn open_about_page(app: AppHandle) {
    open_about(&app);
}

#[tauri::command]
fn hide_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
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
        _ => return Err("Unknown setting".into()),
    }
    save_settings(&app, &s)
}

#[tauri::command]
fn get_today_info() -> DayInfo {
    current_day_info()
}

// ----- Vietnamese lunar calendar (Ho Ngoc Duc / Meeus style, TZ +7) -----

fn int_floor(x: f64) -> i32 { x.floor() as i32 }

fn jd_from_date(dd: i32, mm: i32, yy: i32) -> i32 {
    let a = int_floor((14 - mm) as f64 / 12.0);
    let y = yy + 4800 - a;
    let m = mm + 12 * a - 3;
    let mut jd = dd + int_floor((153 * m + 2) as f64 / 5.0)
        + 365 * y + int_floor(y as f64 / 4.0) - int_floor(y as f64 / 100.0)
        + int_floor(y as f64 / 400.0) - 32045;
    if jd < 2299161 {
        jd = dd + int_floor((153 * m + 2) as f64 / 5.0) + 365 * y
            + int_floor(y as f64 / 4.0) - 32083;
    }
    jd
}

fn new_moon(k: i32) -> f64 {
    let kf = k as f64;
    let t = kf / 1236.85;
    let t2 = t * t;
    let t3 = t2 * t;
    let dr = std::f64::consts::PI / 180.0;
    let mut jd1 = 2415020.75933 + 29.53058868 * kf + 0.0001178 * t2 - 0.000000155 * t3;
    jd1 += 0.00033 * ((166.56 + 132.87 * t - 0.009173 * t2) * dr).sin();
    let m = 359.2242 + 29.10535608 * kf - 0.0000333 * t2 - 0.00000347 * t3;
    let mpr = 306.0253 + 385.81691806 * kf + 0.0107306 * t2 + 0.00001236 * t3;
    let f = 21.2964 + 390.67050646 * kf - 0.0016528 * t2 - 0.00000239 * t3;
    let mut c1 = (0.1734 - 0.000393 * t) * (m * dr).sin()
        + 0.0021 * (2.0 * m * dr).sin()
        - 0.4068 * (mpr * dr).sin()
        + 0.0161 * (2.0 * mpr * dr).sin()
        - 0.0004 * (3.0 * mpr * dr).sin()
        + 0.0104 * (2.0 * f * dr).sin()
        - 0.0051 * ((m + mpr) * dr).sin()
        - 0.0074 * ((m - mpr) * dr).sin()
        + 0.0004 * ((2.0 * f + m) * dr).sin()
        - 0.0004 * ((2.0 * f - m) * dr).sin()
        - 0.0006 * ((2.0 * f + mpr) * dr).sin()
        + 0.0010 * ((2.0 * f - mpr) * dr).sin()
        + 0.0005 * ((2.0 * mpr + m) * dr).sin();
    let deltat = if t < -11.0 {
        0.001 + 0.000839 * t + 0.0002261 * t2 - 0.00000845 * t3 - 0.000000081 * t * t3
    } else {
        -0.000278 + 0.000265 * t + 0.000262 * t2
    };
    jd1 + c1 - deltat
}

fn get_new_moon_day(k: i32, time_zone: f64) -> i32 {
    int_floor(new_moon(k) + 0.5 + time_zone / 24.0)
}

fn sun_longitude(jdn: f64) -> f64 {
    let t = (jdn - 2451545.0) / 36525.0;
    let t2 = t * t;
    let dr = std::f64::consts::PI / 180.0;
    let m = 357.52910 + 35999.05030 * t - 0.0001559 * t2 - 0.00000048 * t * t2;
    let l0 = 280.46645 + 36000.76983 * t + 0.0003032 * t2;
    let dl = (1.914600 - 0.004817 * t - 0.000014 * t2) * (dr * m).sin()
        + (0.019993 - 0.000101 * t) * (dr * 2.0 * m).sin()
        + 0.000290 * (dr * 3.0 * m).sin();
    let mut l = (l0 + dl) * dr;
    l -= std::f64::consts::PI * 2.0 * (l / (std::f64::consts::PI * 2.0)).floor();
    l
}

fn get_sun_longitude(day_number: i32, time_zone: f64) -> i32 {
    int_floor(sun_longitude(day_number as f64 - 0.5 - time_zone / 24.0) / std::f64::consts::PI * 6.0)
}

fn get_lunar_month11(yy: i32, tz: f64) -> i32 {
    let off = jd_from_date(31, 12, yy) as f64 - 2415021.0;
    let k = int_floor(off / 29.530588853);
    let mut nm = get_new_moon_day(k, tz);
    let sun_long = get_sun_longitude(nm, tz);
    if sun_long >= 9 { nm = get_new_moon_day(k - 1, tz); }
    nm
}

fn get_leap_month_offset(a11: i32, tz: f64) -> i32 {
    let k = int_floor((a11 as f64 - 2415021.076998695) / 29.530588853 + 0.5);
    let mut last = 0;
    let mut i = 1;
    let mut arc = get_sun_longitude(get_new_moon_day(k + i, tz), tz);
    loop {
        let last_arc = arc;
        last = last_arc;
        i += 1;
        arc = get_sun_longitude(get_new_moon_day(k + i, tz), tz);
        if arc == last || i >= 14 { break; }
    }
    i - 1
}

#[derive(Debug, Clone)]
struct LunarDate { day: i32, month: i32, year: i32, leap: bool, jd: i32 }

fn solar_to_lunar(dd: i32, mm: i32, yy: i32) -> LunarDate {
    let tz = 7.0;
    let day_number = jd_from_date(dd, mm, yy);
    let k = int_floor((day_number as f64 - 2415021.076998695) / 29.530588853);
    let mut month_start = get_new_moon_day(k + 1, tz);
    if month_start > day_number { month_start = get_new_moon_day(k, tz); }
    let mut a11 = get_lunar_month11(yy, tz);
    let mut b11 = a11;
    let lunar_year;
    if a11 >= month_start {
        lunar_year = yy;
        a11 = get_lunar_month11(yy - 1, tz);
    } else {
        lunar_year = yy + 1;
        b11 = get_lunar_month11(yy + 1, tz);
    }
    let lunar_day = day_number - month_start + 1;
    let diff = int_floor((month_start - a11) as f64 / 29.0);
    let mut lunar_month = diff + 11;
    let mut lunar_leap = false;
    if b11 - a11 > 365 {
        let leap_diff = get_leap_month_offset(a11, tz);
        if diff >= leap_diff {
            lunar_month = diff + 10;
            if diff == leap_diff { lunar_leap = true; }
        }
    }
    if lunar_month > 12 { lunar_month -= 12; }
    let mut ly = lunar_year;
    if lunar_month >= 11 && diff < 4 { ly -= 1; }
    LunarDate { day: lunar_day, month: lunar_month, year: ly, leap: lunar_leap, jd: day_number }
}

fn can_chi_year(year: i32) -> String {
    let can = ["Canh","Tân","Nhâm","Quý","Giáp","Ất","Bính","Đinh","Mậu","Kỷ"];
    let chi = ["Thân","Dậu","Tuất","Hợi","Tý","Sửu","Dần","Mão","Thìn","Tỵ","Ngọ","Mùi"];
    format!("{} {}", can[year.rem_euclid(10) as usize], chi[year.rem_euclid(12) as usize])
}

fn can_chi_day(jd: i32) -> String {
    let can = ["Giáp","Ất","Bính","Đinh","Mậu","Kỷ","Canh","Tân","Nhâm","Quý"];
    let chi = ["Tý","Sửu","Dần","Mão","Thìn","Tỵ","Ngọ","Mùi","Thân","Dậu","Tuất","Hợi"];
    format!("{} {}", can[(jd + 9).rem_euclid(10) as usize], chi[(jd + 1).rem_euclid(12) as usize])
}

fn feng_shui(jd: i32) -> (String, String) {
    let chi_index = (jd + 1).rem_euclid(12);
    let hours: [&[&str]; 12] = [
        &["Tý 23–01","Sửu 01–03","Mão 05–07","Ngọ 11–13","Thân 15–17","Dậu 17–19"],
        &["Dần 03–05","Mão 05–07","Tỵ 09–11","Thân 15–17","Tuất 19–21","Hợi 21–23"],
        &["Tý 23–01","Sửu 01–03","Thìn 07–09","Tỵ 09–11","Mùi 13–15","Tuất 19–21"],
        &["Tý 23–01","Dần 03–05","Mão 05–07","Ngọ 11–13","Mùi 13–15","Dậu 17–19"],
        &["Dần 03–05","Thìn 07–09","Tỵ 09–11","Thân 15–17","Dậu 17–19","Hợi 21–23"],
        &["Sửu 01–03","Thìn 07–09","Ngọ 11–13","Mùi 13–15","Tuất 19–21","Hợi 21–23"],
        &["Tý 23–01","Sửu 01–03","Mão 05–07","Ngọ 11–13","Thân 15–17","Dậu 17–19"],
        &["Dần 03–05","Mão 05–07","Tỵ 09–11","Thân 15–17","Tuất 19–21","Hợi 21–23"],
        &["Tý 23–01","Sửu 01–03","Thìn 07–09","Tỵ 09–11","Mùi 13–15","Tuất 19–21"],
        &["Tý 23–01","Dần 03–05","Mão 05–07","Ngọ 11–13","Mùi 13–15","Dậu 17–19"],
        &["Dần 03–05","Thìn 07–09","Tỵ 09–11","Thân 15–17","Dậu 17–19","Hợi 21–23"],
        &["Sửu 01–03","Thìn 07–09","Ngọ 11–13","Mùi 13–15","Tuất 19–21","Hợi 21–23"],
    ];
    let stem = (jd + 9).rem_euclid(10);
    let hy = ["Đông Bắc","Tây Bắc","Tây Nam","Chính Nam","Đông Nam","Đông Bắc","Tây Bắc","Tây Nam","Chính Nam","Đông Nam"];
    (hours[chi_index as usize].iter().take(3).copied().collect::<Vec<_>>().join(" · "), hy[stem as usize].to_string())
}

fn current_day_info() -> DayInfo {
    let now = Local::now();
    let date = NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap();
    let lunar = solar_to_lunar(now.day() as i32, now.month() as i32, now.year());
    let weekday = match date.weekday().num_days_from_monday() {
        0 => "Thứ Hai", 1 => "Thứ Ba", 2 => "Thứ Tư", 3 => "Thứ Năm",
        4 => "Thứ Sáu", 5 => "Thứ Bảy", _ => "Chủ Nhật"
    }.to_string();
    let leap = if lunar.leap { " nhuận" } else { "" };
    let (good_hours, good_direction) = feng_shui(lunar.jd);
    let event = if now.month() == 9 && now.day() == 2 {
        Some("🇻🇳 Ngày Quốc Khánh Việt Nam".to_string())
    } else { None };

    DayInfo {
        day: now.day(),
        month: now.month(),
        year: now.year(),
        weekday,
        solar: format!("{:02}/{:02}/{}", now.day(), now.month(), now.year()),
        lunar: format!("Âm {}/{}{}", lunar.day, lunar.month, leap),
        lunar_year_can_chi: can_chi_year(lunar.year),
        day_can_chi: can_chi_day(lunar.jd),
        good_hours,
        good_direction,
        event,
    }
}

fn build_tray<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("about", "ℹ️ About LịchAI")
        .text("open_home", "🌐 Mở LịchAI")
        .separator()
        .text("settings", "⚙️ Cài đặt")
        .build()?;

    let info = current_day_info();
    let tooltip = format!("LịchAI • {} • {} • {}", info.solar, info.lunar, info.lunar_year_can_chi);

    TrayIconBuilder::with_id("lichai-tray")
        .tooltip(tooltip)
        .icon(app.default_window_icon().expect("missing app icon").clone())
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "about" => open_about(app),
            "open_home" => open_home(app),
            "settings" => show_mode(app, "settings"),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event {
                show_mode(tray.app_handle(), "today");
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
            open_about_page,
            hide_main_window,
            quit_app,
            autostart_status,
            set_autostart,
            get_settings,
            set_setting,
            get_today_info
        ])
        .setup(|app| {
            build_tray(app)?;
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
