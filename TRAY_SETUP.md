# macOS 菜单栏应用设置完成

## ✅ 已完成的配置

### 1. 系统托盘（菜单栏）图标
- ✅ 在 macOS 顶部菜单栏显示应用图标
- ✅ 点击图标可以显示/隐藏窗口
- ✅ 右键菜单包含：
  - 显示/隐藏
  - 退出

### 2. 隐藏 Dock 图标
- ✅ 设置 `LSUIElement: true` - 不在 Dock 显示
- ✅ 设置 `skipTaskbar: true` - 不在任务栏显示
- ✅ 窗口初始状态为隐藏 `visible: false`

### 3. 托盘功能
- ✅ 左键点击：显示/隐藏窗口
- ✅ 右键菜单：
  - "显示/隐藏" - 切换窗口
  - "退出" - 退出应用

## 📝 配置详情

### tauri.conf.json
```json
{
  "app": {
    "macOSPrivateApi": true,
    "windows": [{
      "visible": false,        // 初始隐藏
      "skipTaskbar": true      // 不在任务栏显示
    }],
    "trayIcon": {
      "id": "main-tray",
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true,  // macOS 模板图标（自动适配深色/浅色模式）
      "title": "My uTools"
    }
  },
  "bundle": {
    "macOS": {
      "minimumSystemVersion": "10.13"
    }
  }
}
```

### lib.rs
```rust
// 创建托盘菜单
let show_item = MenuItemBuilder::with_id("show", "显示/隐藏").build(app)?;
let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

let menu = MenuBuilder::new(app)
    .item(&show_item)
    .separator()
    .item(&quit_item)
    .build()?;

// 创建托盘图标
let _tray = TrayIconBuilder::new()
    .menu(&menu)
    .icon(app.default_window_icon().unwrap().clone())
    .on_menu_event(|app, event| {
        match event.id().as_ref() {
            "show" => toggle_window_visibility(app.clone()),
            "quit" => app.exit(0),
            _ => {}
        }
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click { button, .. } = event {
            if button == tauri::tray::MouseButton::Left {
                toggle_window_visibility(tray.app_handle().clone());
            }
        }
    })
    .build(app)?;
```

## 🎯 使用方式

### 启动应用
```bash
pnpm tauri dev
```

### 预期行为
1. ✅ 应用启动后，**不会**在 Dock 显示图标
2. ✅ 在顶部菜单栏（状态栏）显示应用图标
3. ✅ 窗口初始状态为隐藏
4. ✅ 按 `Alt+Space` 或点击托盘图标显示窗口
5. ✅ 窗口失去焦点自动隐藏
6. ✅ 右键托盘图标显示菜单

### 交互方式
- **显示窗口**：
  - 按 `Alt+Space` 快捷键
  - 点击菜单栏图标
  - 右键菜单 → "显示/隐藏"

- **隐藏窗口**：
  - 按 `Esc` 键
  - 点击窗口外部（失去焦点）
  - 再次按 `Alt+Space`

- **退出应用**：
  - 右键菜单栏图标 → "退出"

## 🎨 图标说明

### 当前图标
- 使用 `icons/icon.png`
- `iconAsTemplate: true` - 自动适配系统主题
  - 浅色模式：深色图标
  - 深色模式：浅色图标

### 自定义图标
如果需要自定义托盘图标：
1. 替换 `src-tauri/icons/icon.png`
2. 建议尺寸：32x32 或 64x64 像素
3. 使用黑白图标（模板模式会自动着色）

## 🔧 故障排除

### 问题：Dock 仍然显示图标
- 确认 `LSUIElement: true` 已设置
- 完全退出应用后重新启动
- 清理构建缓存：`pnpm tauri build --debug`

### 问题：托盘图标不显示
- 检查 `icons/icon.png` 是否存在
- 查看终端日志是否有错误
- 确认 `tray-icon` feature 已启用

### 问题：点击托盘图标无反应
- 检查 `on_tray_icon_event` 是否正确设置
- 查看终端日志

## 📚 参考资料

- [Tauri Tray Icon 文档](https://tauri.app/v2/guides/features/system-tray/)
- [macOS LSUIElement 说明](https://developer.apple.com/documentation/bundleresources/information_property_list/lsuielement)
- [Tauri Menu 文档](https://tauri.app/v2/guides/features/menu/)

## 🎉 完成！

现在你的应用就像 uTools 一样：
- ✅ 只在菜单栏显示
- ✅ 不在 Dock 显示
- ✅ 快捷键快速唤醒
- ✅ 失去焦点自动隐藏
- ✅ 完美的菜单栏应用体验！
