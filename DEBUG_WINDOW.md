# 窗口显示调试指南

## 🔧 已修复的问题

1. **简化了 AppState 结构体**
   - 移除了 `last_show_time` 字段
   - 只保留必要的 `current_shortcut`

2. **添加了调试日志**
   - 窗口切换时会输出详细日志
   - 可以在终端查看窗口状态

## 🚀 重启测试步骤

### 1. 完全退出当前应用
```bash
# 如果应用正在运行，按 Ctrl+C 停止
# 或者右键托盘图标 → 退出
```

### 2. 清理并重新启动
```bash
# 清理构建缓存（可选）
rm -rf src-tauri/target/debug

# 重新启动
pnpm tauri dev
```

### 3. 查看启动日志
应该看到类似的输出：
```
My uTools 启动成功！
系统托盘图标创建成功
全局快捷键 Alt+Space 注册成功
```

### 4. 测试快捷键
按 `Alt+Space`，终端应该输出：
```
Toggle window visibility called
Window is currently visible: false
Showing window
Window shown and focused
```

### 5. 再次按快捷键
应该输出：
```
Toggle window visibility called
Window is currently visible: true
Hiding window
```

## 🐛 故障排除

### 问题 1：按快捷键没反应
**检查：**
1. 查看终端是否有 "Toggle window visibility called" 日志
2. 如果没有，说明快捷键没注册成功

**解决：**
```bash
# 检查是否有其他应用占用了 Alt+Space
# 尝试修改快捷键（在设置中）
```

### 问题 2：有日志但窗口不显示
**检查终端日志：**
- 如果看到 "ERROR: Window 'main' not found!"
  - 说明窗口创建失败
  - 检查 tauri.conf.json 配置

- 如果看到 "Window shown and focused"
  - 但窗口还是不显示
  - 可能是窗口在屏幕外或被其他窗口遮挡

**解决：**
```bash
# 方法 1：重置窗口位置
# 删除配置文件
rm -rf ~/.local/share/com.lewen.my-utools/

# 方法 2：检查窗口配置
# 确保 tauri.conf.json 中：
# - "visible": false (初始隐藏)
# - "center": true (居中显示)
# - "transparent": true (透明背景)
```

### 问题 3：窗口显示但是黑屏
**原因：** 前端加载失败

**检查：**
1. 查看浏览器控制台（右键 → 检查元素）
2. 查看是否有 JavaScript 错误

**解决：**
```bash
# 重新安装依赖
pnpm install

# 清理缓存
rm -rf node_modules/.vite

# 重启
pnpm tauri dev
```

### 问题 4：窗口显示但没有毛玻璃效果
**原因：** 透明窗口可能需要特殊配置

**检查 tauri.conf.json：**
```json
{
  "windows": [{
    "transparent": true,  // 必须为 true
    "decorations": false  // 必须为 false
  }]
}
```

**检查 CSS：**
```css
.app-container {
  background: rgba(30, 30, 30, 0.85);  // 半透明背景
  backdrop-filter: blur(40px);          // 毛玻璃效果
}
```

## 📊 调试日志说明

### 正常启动日志
```
My uTools 启动成功！
系统托盘图标创建成功
全局快捷键 Alt+Space 注册成功
```

### 窗口切换日志
```
Toggle window visibility called
Window is currently visible: false
Showing window
Window shown and focused
```

### 剪贴板监控日志
```
Adding clipboard item: type=text, content_len=20
Clipboard item added successfully. Total items: 1
```

## 🎯 预期行为

### 1. 启动应用
- ✅ 托盘图标出现在菜单栏
- ✅ Dock 不显示图标
- ✅ 窗口初始隐藏
- ✅ 终端显示启动成功日志

### 2. 按 Alt+Space
- ✅ 窗口从屏幕中央淡入
- ✅ 窗口获得焦点
- ✅ 搜索框自动聚焦
- ✅ 终端显示窗口显示日志

### 3. 再按 Alt+Space
- ✅ 窗口淡出隐藏
- ✅ 终端显示窗口隐藏日志

### 4. 点击托盘图标
- ✅ 窗口切换显示/隐藏
- ✅ 行为与快捷键一致

## 🔍 高级调试

### 查看窗口状态
在 Rust 代码中添加更多日志：
```rust
println!("Window position: {:?}", window.outer_position());
println!("Window size: {:?}", window.outer_size());
println!("Window visible: {:?}", window.is_visible());
println!("Window focused: {:?}", window.is_focused());
```

### 查看前端状态
在浏览器控制台中：
```javascript
// 检查 Vue 应用是否正常
console.log(document.querySelector('#app'));

// 检查样式是否加载
console.log(getComputedStyle(document.querySelector('.app-container')));
```

## ✅ 成功标志

如果一切正常，你应该看到：
1. ✅ 终端有完整的启动日志
2. ✅ 按快捷键有窗口切换日志
3. ✅ 窗口能正常显示和隐藏
4. ✅ 窗口有毛玻璃效果
5. ✅ 所有交互都流畅

## 🎉 完成！

如果所有测试都通过，说明窗口显示功能已经正常工作！

现在可以享受你的 macOS 风格应用了！🚀
