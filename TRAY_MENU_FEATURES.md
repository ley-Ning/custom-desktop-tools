# 托盘菜单功能说明

## ✅ 完整的托盘菜单

右键点击菜单栏图标，显示以下菜单项：

### 📋 菜单项列表

1. **My uTools 官网**
   - 打开项目 GitHub 页面
   - 可以自定义为你的官网地址

2. **隐私政策**
   - 打开隐私政策页面
   - 链接到 PRIVACY.md

3. **用户协议**
   - 打开用户协议页面
   - 链接到 TERMS.md

4. **版本 (v0.1.0)**
   - 显示当前应用版本
   - 自动从 package.json 读取

5. **检测更新**
   - 检查是否有新版本
   - TODO: 需要实现更新检测逻辑

6. **设置**
   - 打开设置界面
   - 自动显示窗口并切换到设置页面

7. **显示 / 隐藏**
   - 切换主窗口显示/隐藏
   - 等同于快捷键 `Alt+Space`

8. **重启**
   - 重启应用
   - 保留所有设置和数据

9. **退出**
   - 完全退出应用

## 🎯 功能实现

### Rust 后端 (lib.rs)

```rust
// 创建托盘菜单
let website_item = MenuItemBuilder::with_id("website", "My uTools 官网").build(app)?;
let privacy_item = MenuItemBuilder::with_id("privacy", "隐私政策").build(app)?;
let terms_item = MenuItemBuilder::with_id("terms", "用户协议").build(app)?;

let version = app.package_info().version.to_string();
let version_item = MenuItemBuilder::with_id("version", format!("版本 (v{})", version)).build(app)?;
let update_item = MenuItemBuilder::with_id("update", "检测更新").build(app)?;

let settings_item = MenuItemBuilder::with_id("settings", "设置").build(app)?;
let show_item = MenuItemBuilder::with_id("show", "显示 / 隐藏").build(app)?;
let restart_item = MenuItemBuilder::with_id("restart", "重启").build(app)?;
let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

// 菜单事件处理
.on_menu_event(|app, event| {
    match event.id().as_ref() {
        "website" => {
            // 打开官网
            let _ = tauri::api::shell::open(&app.shell(), "https://github.com/...", None);
        }
        "settings" => {
            // 打开设置界面
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.emit("open-settings", ());
            }
        }
        "restart" => {
            app.restart();
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
})
```

### Vue 前端 (App.vue)

```typescript
// 监听打开设置事件
unlistenOpenSettings = await listen<void>("open-settings", () => {
  openSettings();
});
```

## 🔧 自定义配置

### 修改链接地址

在 `src-tauri/src/lib.rs` 中修改：

```rust
"website" => {
    // 修改为你的官网地址
    let _ = tauri::api::shell::open(&app.shell(), "https://your-website.com", None);
}
"privacy" => {
    // 修改为你的隐私政策地址
    let _ = tauri::api::shell::open(&app.shell(), "https://your-website.com/privacy", None);
}
"terms" => {
    // 修改为你的用户协议地址
    let _ = tauri::api::shell::open(&app.shell(), "https://your-website.com/terms", None);
}
```

### 添加新菜单项

1. **创建菜单项**
```rust
let new_item = MenuItemBuilder::with_id("new_feature", "新功能").build(app)?;
```

2. **添加到菜单**
```rust
let menu = MenuBuilder::new(app)
    .item(&new_item)
    // ... 其他菜单项
    .build()?;
```

3. **处理事件**
```rust
.on_menu_event(|app, event| {
    match event.id().as_ref() {
        "new_feature" => {
            // 处理新功能
        }
        _ => {}
    }
})
```

## 📝 菜单结构

```
┌─────────────────────────┐
│ My uTools 官网          │
├─────────────────────────┤
│ 隐私政策                │
│ 用户协议                │
├─────────────────────────┤
│ 版本 (v0.1.0)          │
│ 检测更新                │
├─────────────────────────┤
│ 设置                    │
│ 显示 / 隐藏             │
├─────────────────────────┤
│ 重启                    │
│ 退出                    │
└─────────────────────────┘
```

## 🎨 使用体验

### 左键点击托盘图标
- 显示/隐藏主窗口
- 快速访问应用

### 右键点击托盘图标
- 显示完整菜单
- 访问所有功能

### 快捷键
- `Alt+Space` - 显示/隐藏窗口
- `Esc` - 隐藏窗口

## 🚀 测试方法

1. **启动应用**
   ```bash
   pnpm tauri dev
   ```

2. **查看托盘图标**
   - 应该在顶部菜单栏显示

3. **测试左键点击**
   - 点击图标 → 窗口显示
   - 再次点击 → 窗口隐藏

4. **测试右键菜单**
   - 右键点击图标
   - 应该看到完整的菜单列表
   - 测试每个菜单项

5. **测试功能**
   - 点击"设置" → 应该打开设置界面
   - 点击"显示/隐藏" → 窗口切换
   - 点击"重启" → 应用重启
   - 点击"退出" → 应用退出

## 📚 相关文件

- `src-tauri/src/lib.rs` - 托盘菜单实现
- `src/App.vue` - 前端事件监听
- `src-tauri/tauri.conf.json` - 托盘配置

## 🎉 完成！

现在你的应用拥有完整的托盘菜单功能，就像 uTools 一样专业！
