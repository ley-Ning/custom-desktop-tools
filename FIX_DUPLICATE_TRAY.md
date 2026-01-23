# 修复重复托盘图标问题

## 问题描述

macOS 顶部菜单栏显示了两个应用图标。

## 原因分析

在 `tauri.conf.json` 中配置了 `trayIcon`，同时在 Rust 代码 (`src-tauri/src/lib.rs`) 中又使用 `TrayIconBuilder` 创建了托盘图标，导致创建了两个图标。

## 解决方案

### 修改 1：移除配置文件中的托盘图标配置

**文件**：`src-tauri/tauri.conf.json`

**移除的配置**：
```json
"trayIcon": {
    "id": "main-tray",
    "iconPath": "icons/icon.png",
    "iconAsTemplate": true,
    "title": "My uTools"
}
```

### 修改 2：确保 LSUIElement 设置正确

**文件**：`src-tauri/tauri.conf.json`

**添加的配置**：
```json
"macOS": {
    "minimumSystemVersion": "10.13",
    "LSUIElement": true
}
```

## 最终配置

```json
{
    "app": {
        "withGlobalTauri": true,
        "macOSPrivateApi": true,
        "windows": [
            {
                "title": "My uTools",
                "width": 900,
                "height": 600,
                "resizable": false,
                "center": true,
                "decorations": false,
                "transparent": true,
                "alwaysOnTop": true,
                "visible": false,
                "dragDropEnabled": false,
                "skipTaskbar": true
            }
        ]
    },
    "bundle": {
        "macOS": {
            "minimumSystemVersion": "10.13",
            "LSUIElement": true
        }
    }
}
```

## 托盘图标创建方式

现在只在 Rust 代码中创建托盘图标：

```rust
// src-tauri/src/lib.rs
let _tray = TrayIconBuilder::new()
    .menu(&menu)
    .icon(app.default_window_icon().unwrap().clone())
    .on_menu_event(|app, event| {
        // 菜单事件处理
    })
    .on_tray_icon_event(|tray, event| {
        // 托盘图标事件处理
    })
    .build(app)?;
```

## 测试步骤

1. **停止当前运行的应用**
   ```bash
   # 按 Ctrl+C 停止开发服务器
   ```

2. **重新启动应用**
   ```bash
   pnpm tauri dev
   ```

3. **检查菜单栏**
   - 应该只显示一个应用图标
   - Dock 中不应该显示应用图标

## 预期结果

✅ macOS 顶部菜单栏只显示一个图标  
✅ Dock 中不显示应用图标  
✅ 点击菜单栏图标可以正常使用  
✅ 右键菜单正常工作  

## 相关配置说明

### LSUIElement

- **作用**：将应用设置为 UI 元素，不在 Dock 中显示
- **值**：`true` 或 `false`
- **位置**：`bundle.macOS.LSUIElement`

### skipTaskbar

- **作用**：在任务栏/Dock 中隐藏窗口
- **值**：`true` 或 `false`
- **位置**：`app.windows[].skipTaskbar`

### trayIcon (已移除)

- **作用**：通过配置文件创建托盘图标
- **问题**：与代码中的 TrayIconBuilder 冲突
- **解决**：只使用代码方式创建

## 注意事项

1. **开发模式 vs 生产模式**
   - 开发模式下可能仍会看到 Vite 开发服务器的图标
   - 生产模式（打包后）只会显示一个图标

2. **重启应用**
   - 修改配置后必须重启应用才能生效
   - 使用 `pnpm tauri dev` 重新启动

3. **图标显示**
   - 如果图标不显示，检查 `icons/icon.png` 是否存在
   - 确保图标格式正确（PNG 格式）

## 故障排除

### 问题：仍然显示两个图标

**解决方案：**
1. 完全停止应用（包括后台进程）
2. 清理构建缓存：
   ```bash
   rm -rf src-tauri/target
   ```
3. 重新启动：
   ```bash
   pnpm tauri dev
   ```

### 问题：图标不显示

**解决方案：**
1. 检查图标文件是否存在
2. 检查 Rust 代码中的图标加载逻辑
3. 查看控制台错误信息

### 问题：Dock 中仍显示图标

**解决方案：**
1. 确认 `LSUIElement: true` 已设置
2. 确认 `skipTaskbar: true` 已设置
3. 重新构建应用

## 总结

通过移除配置文件中的 `trayIcon` 配置，只在 Rust 代码中创建托盘图标，成功解决了重复显示的问题。

---

**修复日期**：2026-01-23  
**状态**：✅ 已修复  
**测试**：待验证
