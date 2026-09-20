use std::{fs, path::Path, process::Command, sync::Arc};
use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Emitter, Manager, State, Window};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;
use std::sync::Mutex;
use base64::prelude::*;

mod ai_chat;
mod translator;
mod calc;
mod text_tools;

/// 应用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppInfo {
    id: String,
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
}

/// 全局状态
#[derive(Clone)]
struct AppState {
    current_shortcut: Arc<Mutex<Option<String>>>,
}

/// 从 macOS 应用获取图标（使用 sips 快速提取）
fn get_app_icon(app_path: &Path) -> Option<String> {
    let app_name = app_path.file_stem()?.to_str()?;
    let contents_path = app_path.join("Contents");
    let resources_path = contents_path.join("Resources");

    // 如果 Resources 目录不存在，返回 None
    if !resources_path.exists() {
        return None;
    }

    // 1. 先检查常见的图标文件名
    let app_icns = format!("{}.icns", app_name);
    let common_names = ["AppIcon.icns", "app.icns", "App-icon.icns", &app_icns];

    let icns_path = common_names.into_iter()
        .find_map(|name| {
            let path = resources_path.join(name);
            if path.exists() { Some(path) } else { None }
        });

    // 2. 如果常见名称找不到，遍历 Resources 目录找第一个 .icns 文件
    let icns_path = match icns_path {
        Some(p) => p,
        None => {
            fs::read_dir(&resources_path).ok()?
                .flatten()
                .find(|entry| {
                    entry.path().extension().and_then(|s| s.to_str()) == Some("icns")
                })
                .map(|e| e.path())?
        }
    };

    // 临时文件路径
    let temp_dir = std::env::temp_dir();
    let temp_png = temp_dir.join(format!("{}_icon.png", uuid::Uuid::new_v4()));

    // 使用 sips 转换 icns 到 png（比 qlmanage 快很多）
    let result = Command::new("sips")
        .args([
            "-s", "format", "png",
            "-z", "64", "64",
            &icns_path.to_string_lossy(),
            "--out", &temp_png.to_string_lossy()
        ])
        .output()
        .ok()?;

    if !result.status.success() {
        return None;
    }

    let png_data = fs::read(&temp_png).ok()?;
    let base64_string = BASE64_STANDARD.encode(&png_data);

    // 清理临时文件
    let _ = fs::remove_file(&temp_png);

    Some(format!("data:image/png;base64,{}", base64_string))
}

/// 模糊搜索
fn fuzzy_match(keyword: &str, target: &str) -> bool {
    let keyword_lower = keyword.to_lowercase();
    let target_lower = target.to_lowercase();

    if target_lower.contains(&keyword_lower) {
        return true;
    }

    if keyword_lower.chars().count() <= target_lower.chars().count() / 2 {
        let initials: String = target_lower
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .collect();

        if initials.contains(&keyword_lower) {
            return true;
        }
    }

    false
}

/// 扫描系统应用目录
#[command]
async fn get_apps(query: Option<String>) -> Vec<AppInfo> {
    let mut apps = Vec::new();
    
    // 扫描多个应用目录
    let app_dirs = vec![
        "/Applications",
        "/System/Applications",
        "/System/Applications/Utilities",
    ];

    for apps_dir in app_dirs {
        let dir_path = Path::new(apps_dir);
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("app") {
                    if let Some(app_name) = path.file_stem().and_then(|s| s.to_str()) {
                        let icon = get_app_icon(&path);
                        apps.push(AppInfo {
                            id: app_name.to_string(),
                            name: app_name.to_string(),
                            path: path.to_string_lossy().to_string(),
                            icon,
                        });
                    }
                }
            }
        }
    }

    if let Some(keyword) = query {
        if !keyword.is_empty() {
            return apps
                .into_iter()
                .filter(|app| fuzzy_match(&keyword, &app.name))
                .collect();
        }
    }

    apps
}

/// 启动应用
#[command]
async fn launch_app(path: String, window: Window) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let result = Command::new("open")
            .arg("-a")
            .arg(&path)
            .spawn();

        match result {
            Ok(_) => {
                let _ = window.hide();
                Ok(())
            }
            Err(e) => Err(format!("启动应用失败: {}", e)),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(format!("不支持的平台"))
    }
}

/// 获取当前快捷键
#[command]
async fn get_shortcut(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state.current_shortcut.lock().unwrap().clone().unwrap_or("Alt+Space".to_string()))
}

/// 隐藏窗口
#[command]
async fn hide_window(window: Window) {
    let _ = window.hide();
}

/// 设置快捷键
#[command]
async fn set_shortcut(
    shortcut: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (modifiers, key) = parse_shortcut(&shortcut)?;
    let new_shortcut = Shortcut::new(modifiers, key);

    // 注销旧的快捷键
    let old_shortcut = state.current_shortcut.lock().unwrap().clone();
    drop(state.current_shortcut.lock().unwrap());

    if let Some(old_str) = old_shortcut {
        if let Ok((old_mods, old_key)) = parse_shortcut(&old_str) {
            let old_sc = Shortcut::new(old_mods, old_key);
            let _ = app.global_shortcut().unregister(old_sc);
        }
    }

    // 注册新的快捷键
    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut(new_shortcut, move |_app, _sc, event| {
            // 只在按键按下时触发，松开时不触发
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                toggle_window_visibility(app_handle.clone());
            }
        })
        .map_err(|e| format!("注册快捷键失败: {}", e))?;

    *state.current_shortcut.lock().unwrap() = Some(shortcut.clone());

    // 保存到 store
    if let Ok(store) = app.store("settings.json") {
        let _ = store.set("shortcut", serde_json::json!(shortcut));
        let _ = store.save();
    }

    Ok(())
}

/// 解析快捷键字符串
fn parse_shortcut(shortcut: &str) -> Result<(Option<Modifiers>, Code), String> {
    let parts: Vec<&str> = shortcut.split('+').collect();
    if parts.len() < 2 {
        return Err(format!("无效的快捷键格式: {}", shortcut));
    }

    let mut modifiers = Modifiers::empty();

    for part in &parts[..parts.len()-1] {
        match *part {
            "Alt" => modifiers |= Modifiers::ALT,
            "CmdOrCtrl" | "Cmd" | "Control" => modifiers |= Modifiers::CONTROL,
            "Shift" => modifiers |= Modifiers::SHIFT,
            "Super" => modifiers |= Modifiers::SUPER,
            _ => return Err(format!("不支持的修饰键: {}", part)),
        }
    }

    let key_str = parts.last().unwrap();
    let key = match *key_str {
        "Space" => Code::Space,
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap();
            match c {
                'A' | 'a' => Code::KeyA,
                'B' | 'b' => Code::KeyB,
                'C' | 'c' => Code::KeyC,
                'D' | 'd' => Code::KeyD,
                'E' | 'e' => Code::KeyE,
                'F' | 'f' => Code::KeyF,
                'G' | 'g' => Code::KeyG,
                'H' | 'h' => Code::KeyH,
                'I' | 'i' => Code::KeyI,
                'J' | 'j' => Code::KeyJ,
                'K' | 'k' => Code::KeyK,
                'L' | 'l' => Code::KeyL,
                'M' | 'm' => Code::KeyM,
                'N' | 'n' => Code::KeyN,
                'O' | 'o' => Code::KeyO,
                'P' | 'p' => Code::KeyP,
                'Q' | 'q' => Code::KeyQ,
                'R' | 'r' => Code::KeyR,
                'S' | 's' => Code::KeyS,
                'T' | 't' => Code::KeyT,
                'U' | 'u' => Code::KeyU,
                'V' | 'v' => Code::KeyV,
                'W' | 'w' => Code::KeyW,
                'X' | 'x' => Code::KeyX,
                'Y' | 'y' => Code::KeyY,
                'Z' | 'z' => Code::KeyZ,
                '0' => Code::Digit0,
                '1' => Code::Digit1,
                '2' => Code::Digit2,
                '3' => Code::Digit3,
                '4' => Code::Digit4,
                '5' => Code::Digit5,
                '6' => Code::Digit6,
                '7' => Code::Digit7,
                '8' => Code::Digit8,
                '9' => Code::Digit9,
                _ => return Err(format!("不支持的按键: {}", s)),
            }
        }
        _ => return Err(format!("不支持的按键: {}", key_str)),
    };

    Ok((
        if modifiers.is_empty() { None } else { Some(modifiers) },
        key
    ))
}

/// 切换窗口可见性
fn toggle_window_visibility(app: AppHandle) {
    println!("Toggle window visibility called");
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        println!("Window is currently visible: {}", is_visible);
        
        if is_visible {
            println!("Hiding window");
            let _ = window.hide();
        } else {
            println!("Showing window");
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.emit("clear-search", ());
            println!("Window shown and focused");
        }
    } else {
        println!("ERROR: Window 'main' not found!");
    }
}

#[command]
async fn toggle_window(app: AppHandle) {
    toggle_window_visibility(app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::{TrayIconBuilder, TrayIconEvent};
    
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            println!("如意 RuYi 启动成功！");

            let app_state = AppState {
                current_shortcut: Arc::new(Mutex::new(None)),
            };
            app.manage(app_state);
            app.manage(ai_chat::AiGeneration::default());

            // 创建托盘菜单
            let website_item = MenuItemBuilder::with_id("website", "如意 RuYi 官网").build(app)?;
            let privacy_item = MenuItemBuilder::with_id("privacy", "隐私政策").build(app)?;
            let terms_item = MenuItemBuilder::with_id("terms", "用户协议").build(app)?;
            
            // 获取版本号
            let version = app.package_info().version.to_string();
            let version_item = MenuItemBuilder::with_id("version", format!("版本 (v{})", version)).build(app)?;
            let update_item = MenuItemBuilder::with_id("update", "检测更新").build(app)?;
            
            let settings_item = MenuItemBuilder::with_id("settings", "设置").build(app)?;
            let show_item = MenuItemBuilder::with_id("show", "显示 / 隐藏").build(app)?;
            let restart_item = MenuItemBuilder::with_id("restart", "重启").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;
            
            let menu = MenuBuilder::new(app)
                .item(&website_item)
                .separator()
                .item(&privacy_item)
                .item(&terms_item)
                .separator()
                .item(&version_item)
                .item(&update_item)
                .separator()
                .item(&settings_item)
                .item(&show_item)
                .separator()
                .item(&restart_item)
                .item(&quit_item)
                .build()?;

            // 创建托盘图标
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "website" => {
                            // 打开官网
                            let _ = app.opener().open_url("https://github.com/ley-Ning/custom-desktop-tools", None::<&str>);
                        }
                        "privacy" => {
                            // 打开隐私政策
                            let _ = app.opener().open_url("https://github.com/ley-Ning/custom-desktop-tools/blob/main/README.md", None::<&str>);
                        }
                        "terms" => {
                            // 打开用户协议
                            let _ = app.opener().open_url("https://github.com/ley-Ning/custom-desktop-tools/blob/main/LICENSE", None::<&str>);
                        }
                        "version" => {
                            // 显示版本信息（可以打开关于页面）
                            println!("当前版本: {}", app.package_info().version);
                        }
                        "update" => {
                            // 打开设置界面并触发检查更新（更新逻辑在前端 Settings 组件）
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("check-update", ());
                            }
                        }
                        "settings" => {
                            // 打开设置界面
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                // 发送事件到前端打开设置
                                let _ = window.emit("open-settings", ());
                            }
                        }
                        "show" => {
                            toggle_window_visibility(app.clone());
                        }
                        "restart" => {
                            // 重启应用
                            app.restart();
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            let app = tray.app_handle();
                            toggle_window_visibility(app.clone());
                        }
                    }
                })
                .build(app)?;

            println!("系统托盘图标创建成功");

            // 从 store 读取保存的快捷键
            let default_shortcut = "Alt+Space".to_string();
            let stored_shortcut = if let Ok(store) = app.store("settings.json") {
                store.get("shortcut").and_then(|v| v.as_str().map(String::from))
            } else {
                None
            };

            let shortcut_to_use = stored_shortcut.unwrap_or(default_shortcut);

            // 注册全局快捷键
            if let Ok((modifiers, key)) = parse_shortcut(&shortcut_to_use) {
                let shortcut = Shortcut::new(modifiers, key);
                let app_handle = app.handle().clone();

                if app.global_shortcut().on_shortcut(shortcut, move |_app, _sc, event| {
                    // 只在按键按下时触发，松开时不触发
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        toggle_window_visibility(app_handle.clone());
                    }
                }).is_err() {
                    eprintln!("注册快捷键 {} 失败", shortcut_to_use);
                } else {
                    println!("全局快捷键 {} 注册成功", shortcut_to_use);
                    if let Some(state) = app.try_state::<AppState>() {
                        *state.current_shortcut.lock().unwrap() = Some(shortcut_to_use);
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_apps,
            launch_app,
            get_shortcut,
            set_shortcut,
            toggle_window,
            hide_window,
            get_clipboard_history,
            add_clipboard_item,
            delete_clipboard_item,
            clear_clipboard_history,
            toggle_clipboard_favorite,
            read_clipboard_text,
            write_clipboard_text,
            get_memos,
            add_memo,
            update_memo,
            delete_memo,
            toggle_memo_pin,
            load_mcp_config,
            save_mcp_config,
            load_ai_config,
            save_ai_config,
            ai_chat::send_ai_message,
            ai_chat::stop_ai_generation,
            ai_chat::load_conversations,
            ai_chat::save_conversation,
            ai_chat::delete_conversation,
            ai_chat::open_external_url,
            translator::translate_text,
            calc::evaluate_expression,
            calc::evaluate_scratchpad,
            calc::load_calc_lines,
            calc::save_calc_lines,
            text_tools::text_hash,
            text_tools::text_base64_encode,
            text_tools::text_base64_decode,
            open_plugin_window,
            call_mcp_tool,
            save_memo_to_yxbj
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


/// 剪贴板历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardItem {
    id: String,
    content: String,
    content_type: String, // "text", "image", "file"
    timestamp: i64,
    favorite: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_size: Option<u64>,
}

/// 备忘录项
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoItem {
    id: String,
    title: String,
    content: String,
    tags: Vec<String>,
    timestamp: i64,
    updated_at: i64,
    pinned: bool,
}

/// 获取剪贴板历史
#[command]
async fn get_clipboard_history(app: AppHandle) -> Result<Vec<ClipboardItem>, String> {
    if let Ok(store) = app.store("clipboard.json") {
        if let Some(history) = store.get("history") {
            if let Ok(items) = serde_json::from_value::<Vec<ClipboardItem>>(history.clone()) {
                return Ok(items);
            }
        }
    }
    Ok(Vec::new())
}

/// 添加剪贴板项
#[command]
async fn add_clipboard_item(
    content: String,
    content_type: String,
    app: AppHandle,
) -> Result<ClipboardItem, String> {
    println!("Adding clipboard item: type={}, content_len={}", content_type, content.len());
    
    let item = ClipboardItem {
        id: uuid::Uuid::new_v4().to_string(),
        content: content.clone(),
        content_type: content_type.clone(),
        timestamp: chrono::Utc::now().timestamp(),
        favorite: false,
        file_path: None,
        file_size: None,
    };

    if let Ok(store) = app.store("clipboard.json") {
        let mut history = if let Some(h) = store.get("history") {
            serde_json::from_value::<Vec<ClipboardItem>>(h.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        // 避免重复（只检查最近的 5 条）
        let is_duplicate = history.iter().take(5).any(|i| i.content == content);
        
        if !is_duplicate {
            history.insert(0, item.clone());
            
            // 限制历史记录数量为 100
            if history.len() > 100 {
                history.truncate(100);
            }

            let _ = store.set("history", serde_json::to_value(&history).unwrap());
            let _ = store.save();
            println!("Clipboard item added successfully. Total items: {}", history.len());
        } else {
            println!("Duplicate clipboard item, skipped");
        }
    }

    Ok(item)
}

/// 删除剪贴板项
#[command]
async fn delete_clipboard_item(id: String, app: AppHandle) -> Result<(), String> {
    if let Ok(store) = app.store("clipboard.json") {
        if let Some(history) = store.get("history") {
            if let Ok(mut items) = serde_json::from_value::<Vec<ClipboardItem>>(history.clone()) {
                items.retain(|item| item.id != id);
                let _ = store.set("history", serde_json::to_value(&items).unwrap());
                let _ = store.save();
            }
        }
    }
    Ok(())
}

/// 清空剪贴板历史
#[command]
async fn clear_clipboard_history(app: AppHandle) -> Result<(), String> {
    if let Ok(store) = app.store("clipboard.json") {
        let _ = store.set("history", serde_json::to_value(Vec::<ClipboardItem>::new()).unwrap());
        let _ = store.save();
    }
    Ok(())
}

/// 切换剪贴板项收藏状态
#[command]
async fn toggle_clipboard_favorite(id: String, app: AppHandle) -> Result<(), String> {
    if let Ok(store) = app.store("clipboard.json") {
        if let Some(history) = store.get("history") {
            if let Ok(mut items) = serde_json::from_value::<Vec<ClipboardItem>>(history.clone()) {
                if let Some(item) = items.iter_mut().find(|i| i.id == id) {
                    item.favorite = !item.favorite;
                }
                let _ = store.set("history", serde_json::to_value(&items).unwrap());
                let _ = store.save();
            }
        }
    }
    Ok(())
}

/// 获取备忘录列表
#[command]
async fn get_memos(app: AppHandle) -> Result<Vec<MemoItem>, String> {
    if let Ok(store) = app.store("memos.json") {
        if let Some(memos) = store.get("memos") {
            if let Ok(items) = serde_json::from_value::<Vec<MemoItem>>(memos.clone()) {
                return Ok(items);
            }
        }
    }
    Ok(Vec::new())
}

/// 添加备忘录
#[command]
async fn add_memo(
    title: String,
    content: String,
    tags: Vec<String>,
    app: AppHandle,
) -> Result<MemoItem, String> {
    let now = chrono::Utc::now().timestamp();
    let item = MemoItem {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        content,
        tags,
        timestamp: now,
        updated_at: now,
        pinned: false,
    };

    if let Ok(store) = app.store("memos.json") {
        let mut memos = if let Some(m) = store.get("memos") {
            serde_json::from_value::<Vec<MemoItem>>(m.clone()).unwrap_or_default()
        } else {
            Vec::new()
        };

        memos.insert(0, item.clone());
        let _ = store.set("memos", serde_json::to_value(&memos).unwrap());
        let _ = store.save();
    }

    Ok(item)
}

/// 更新备忘录
#[command]
async fn update_memo(
    id: String,
    title: String,
    content: String,
    tags: Vec<String>,
    app: AppHandle,
) -> Result<(), String> {
    if let Ok(store) = app.store("memos.json") {
        if let Some(memos) = store.get("memos") {
            if let Ok(mut items) = serde_json::from_value::<Vec<MemoItem>>(memos.clone()) {
                if let Some(item) = items.iter_mut().find(|i| i.id == id) {
                    item.title = title;
                    item.content = content;
                    item.tags = tags;
                    item.updated_at = chrono::Utc::now().timestamp();
                }
                let _ = store.set("memos", serde_json::to_value(&items).unwrap());
                let _ = store.save();
            }
        }
    }
    Ok(())
}

/// 删除备忘录
#[command]
async fn delete_memo(id: String, app: AppHandle) -> Result<(), String> {
    if let Ok(store) = app.store("memos.json") {
        if let Some(memos) = store.get("memos") {
            if let Ok(mut items) = serde_json::from_value::<Vec<MemoItem>>(memos.clone()) {
                items.retain(|item| item.id != id);
                let _ = store.set("memos", serde_json::to_value(&items).unwrap());
                let _ = store.save();
            }
        }
    }
    Ok(())
}

/// 切换备忘录置顶状态
#[command]
async fn toggle_memo_pin(id: String, app: AppHandle) -> Result<(), String> {
    if let Ok(store) = app.store("memos.json") {
        if let Some(memos) = store.get("memos") {
            if let Ok(mut items) = serde_json::from_value::<Vec<MemoItem>>(memos.clone()) {
                if let Some(item) = items.iter_mut().find(|i| i.id == id) {
                    item.pinned = !item.pinned;
                }
                let _ = store.set("memos", serde_json::to_value(&items).unwrap());
                let _ = store.save();
            }
        }
    }
    Ok(())
}


/// 读取剪贴板文本
#[command]
async fn read_clipboard_text() -> Result<String, String> {
    use std::process::Command;
    
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste")
            .output()
            .map_err(|e| format!("Failed to read clipboard: {}", e))?;
        
        let text = String::from_utf8(output.stdout)
            .map_err(|e| format!("Failed to parse clipboard content: {}", e))?;
        
        // 只返回非空文本
        if text.trim().is_empty() {
            return Err("Clipboard is empty".to_string());
        }
        
        Ok(text)
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Err("Clipboard not supported on this platform".to_string())
    }
}

/// 写入剪贴板文本
#[command]
async fn write_clipboard_text(text: String) -> Result<(), String> {
    use std::process::{Command, Stdio};
    use std::io::Write;
    
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to write clipboard: {}", e))?;
        
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())
                .map_err(|e| format!("Failed to write to clipboard: {}", e))?;
        }
        
        child.wait()
            .map_err(|e| format!("Failed to wait for pbcopy: {}", e))?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Err("Clipboard not supported on this platform".to_string())
    }
}

/// 获取 MCP 配置文件路径
fn get_mcp_config_path() -> Result<std::path::PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    let config_dir = home_dir.join(".local/share/com.lewen.my-utools");
    
    // 确保目录存在
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    
    Ok(config_dir.join("mcp.json"))
}

/// 加载 MCP 配置
#[command]
async fn load_mcp_config() -> Result<String, String> {
    let config_path = get_mcp_config_path()?;
    
    if !config_path.exists() {
        // 返回默认配置
        return Ok(r#"{"mcpServers":{}}"#.to_string());
    }
    
    fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))
}

/// 保存 MCP 配置
#[command]
async fn save_mcp_config(config: String) -> Result<(), String> {
    let config_path = get_mcp_config_path()?;
    
    // 验证 JSON 格式
    serde_json::from_str::<serde_json::Value>(&config)
        .map_err(|e| format!("无效的 JSON 格式: {}", e))?;
    
    fs::write(&config_path, config)
        .map_err(|e| format!("保存配置文件失败: {}", e))
}

/// 获取 AI 配置文件路径
pub(crate) fn get_ai_config_path() -> Result<std::path::PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("无法获取用户目录")?;
    let config_dir = home_dir.join(".local/share/com.lewen.my-utools");
    
    // 确保目录存在
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    
    Ok(config_dir.join("ai-config.json"))
}

/// 加载 AI 配置
#[command]
async fn load_ai_config() -> Result<String, String> {
    let config_path = get_ai_config_path()?;
    
    if !config_path.exists() {
        // 返回默认配置
        return Ok(r#"{"models":[]}"#.to_string());
    }
    
    fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))
}

/// 保存 AI 配置
#[command]
async fn save_ai_config(config: String) -> Result<(), String> {
    let config_path = get_ai_config_path()?;
    
    // 验证 JSON 格式
    serde_json::from_str::<serde_json::Value>(&config)
        .map_err(|e| format!("无效的 JSON 格式: {}", e))?;
    
    fs::write(&config_path, config)
        .map_err(|e| format!("保存配置文件失败: {}", e))
}

/// 打开插件独立窗口
#[command]
async fn open_plugin_window(plugin_id: String, app: AppHandle) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    use std::sync::atomic::{AtomicU32, Ordering};

    static WINDOW_COUNTER: AtomicU32 = AtomicU32::new(0);
    let counter = WINDOW_COUNTER.fetch_add(1, Ordering::SeqCst);

    let window_label = format!("plugin-{}-{}", plugin_id, counter);
    let window_title = match plugin_id.as_str() {
        "json" => "JSON 编辑器",
        "clipboard" => "剪贴板历史",
        "memo" => "备忘快贴",
        "ai" => "AI 对话",
        "translator" => "聚合翻译",
        "calc" => "计算稿纸",
        "text" => "文本工具箱",
        _ => "插件窗口",
    };

    // 使用应用自身资源路径（dev 与打包环境均可用的入口），
    // 通过 URL 参数告知前端要打开的插件
    let url = format!("index.html?plugin={}", plugin_id);

    WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::App(url.into()),
    )
    .title(window_title)
    .inner_size(1000.0, 700.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| format!("创建窗口失败: {}", e))?;

    Ok(())
}

/// 调用 MCP 工具
#[command]
async fn call_mcp_tool(
    server_name: String,
    tool_name: String,
    arguments: serde_json::Value,
    _app: AppHandle,
) -> Result<serde_json::Value, String> {
    // 读取 MCP 配置
    let config_str = load_mcp_config().await?;
    let config: serde_json::Value = serde_json::from_str(&config_str)
        .map_err(|e| format!("解析 MCP 配置失败: {}", e))?;
    
    // 获取服务器配置
    let servers = config.get("mcpServers")
        .and_then(|s| s.as_object())
        .ok_or("MCP 配置中没有 mcpServers")?;
    
    let server_config = servers.get(&server_name)
        .ok_or(format!("未找到 MCP 服务器: {}", server_name))?;
    
    let command = server_config.get("command")
        .and_then(|c| c.as_str())
        .ok_or("服务器配置缺少 command")?;
    
    let args = server_config.get("args")
        .and_then(|a| a.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
        .unwrap_or_default();
    
    // 构建 MCP 请求
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    });
    
    // 执行命令
    use std::process::{Command, Stdio};
    use std::io::{Write, BufRead, BufReader};
    
    let mut child = Command::new(command)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动 MCP 服务器失败: {}", e))?;
    
    // 写入请求
    if let Some(mut stdin) = child.stdin.take() {
        let request_str = serde_json::to_string(&request)
            .map_err(|e| format!("序列化请求失败: {}", e))?;
        writeln!(stdin, "{}", request_str)
            .map_err(|e| format!("写入请求失败: {}", e))?;
    }
    
    // 读取响应
    let stdout = child.stdout.take()
        .ok_or("无法获取 stdout")?;
    let reader = BufReader::new(stdout);
    
    for line in reader.lines() {
        let line = line.map_err(|e| format!("读取响应失败: {}", e))?;
        if line.trim().is_empty() {
            continue;
        }
        
        let response: serde_json::Value = serde_json::from_str(&line)
            .map_err(|e| format!("解析响应失败: {}", e))?;
        
        if let Some(error) = response.get("error") {
            return Err(format!("MCP 工具调用失败: {}", error));
        }
        
        if let Some(result) = response.get("result") {
            return Ok(result.clone());
        }
    }
    
    Err("未收到有效响应".to_string())
}

/// 保存笔记到印象笔记
#[command]
async fn save_memo_to_yxbj(
    title: String,
    content: String,
    tags: Vec<String>,
    app: AppHandle,
) -> Result<String, String> {
    // 构建笔记内容（Markdown 格式）
    let mut note_content = format!("# {}\n\n{}", title, content);
    
    if !tags.is_empty() {
        note_content.push_str(&format!("\n\n---\n标签: {}", tags.join(", ")));
    }
    
    // 调用印象笔记 MCP 工具
    let arguments = serde_json::json!({
        "title": title,
        "content": note_content,
        "tags": tags
    });
    
    let result = call_mcp_tool(
        "yxbj-mcp".to_string(),
        "create_note".to_string(),
        arguments,
        app,
    ).await?;
    
    // 提取笔记 ID
    let note_id = result.get("noteId")
        .or_else(|| result.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    Ok(note_id)
}
