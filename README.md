# custom-desktop-tools

一个类 uTools 的应用启动器，使用 Tauri 2 + Vue 3 + TypeScript 构建，专为 macOS 设计。

## 功能特性

- 🚀 **全局快捷键唤醒**：默认 `Alt+Space`，可自定义
- 🔍 **智能搜索**：模糊搜索 + 首字母缩写匹配
- 🎨 **现代化 UI**：仿 uTools 深色主题设计
- ⚡ **快速启动**：键盘导航，一键启动应用
- 📋 **剪贴板历史**：自动记录剪贴板历史，支持搜索和收藏
- 📝 **备忘快贴**：快速记录笔记，支持标签和置顶
- 🔌 **插件市场**：浏览和安装插件
- ⚙️ **完整设置**：快捷键、偏好设置、AI 模型等

## 快捷键

- `Alt+Space` - 唤出/隐藏窗口
- `↑` / `↓` - 导航应用列表
- `Enter` - 启动选中的应用
- `Esc` - 清空搜索 / 隐藏窗口

## 开发

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev

# 构建应用
pnpm tauri build
```

## 技术栈

- **前端**：Vue 3 + TypeScript + Vite
- **后端**：Rust + Tauri 2
- **UI**：macOS 风格深色主题
- **包管理**：pnpm

## 项目结构

```
my-utools/
├── src/                    # Vue 前端代码
│   ├── components/         # Vue 组件
│   ├── styles/            # 全局样式
│   ├── types/             # TypeScript 类型定义
│   ├── db/                # 数据库
│   ├── App.vue            # 主应用组件
│   └── main.ts            # 入口文件
├── src-tauri/             # Tauri 后端代码
│   ├── src/
│   │   ├── lib.rs         # 核心逻辑
│   │   └── main.rs        # 入口文件
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
└── package.json           # Node 依赖
```

## License

MIT
