# 如意 RuYi（custom-desktop-tools）

一个类 uTools 的应用启动器，使用 Tauri 2 + Vue 3 + TypeScript 构建，专为 macOS 设计。产品名「如意 RuYi」——愿你手边的工具皆如心意。

## 功能特性

- 🚀 **全局快捷键唤醒**：默认 `Alt+Space`，可自定义
- 🔍 **智能搜索**：模糊搜索 + 首字母缩写匹配
- 🎨 **现代化 UI**：仿 uTools 深色主题设计
- ⚡ **快速启动**：键盘导航，一键启动应用
- 📋 **剪贴板历史**：自动记录剪贴板历史，支持搜索和收藏
- 📝 **备忘快贴**：快速记录笔记，支持标签和置顶
- 🤖 **AI 对话**：多会话管理、流式响应、Markdown 与代码高亮渲染、系统提示词与上下文长度设置，兼容 OpenAI 协议的任意模型（DeepSeek / OpenRouter / 自定义网关等），Anthropic 官方 API 原生支持
- 🌐 **聚合翻译**：MyMemory / Google / AI 多引擎，自动检测语言、输入即翻译、译文一键复制
- 🧮 **计算稿纸**：主搜索栏直接算（输入 `1+2*3` 即出结果），稿纸式多行计算支持变量、函数与 `ans` 引用
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

# 构建应用（本地构建更新包需设置签名私钥）
TAURI_SIGNING_PRIVATE_KEY=$(cat ~/.tauri/custom-desktop-tools.key) pnpm tauri build
```

## 应用内热更新

应用内置自动更新（设置 → 软件更新 / 托盘 → 检测更新），更新源为 GitHub Releases：

1. 密钥对已生成：私钥 `~/.tauri/custom-desktop-tools.key`、密码
   `~/.tauri/custom-desktop-tools.key.password`，公钥在 `tauri.conf.json`。
   **两者务必保管好——丢失后无法再签发更新**
2. 在仓库 Settings → Secrets → Actions 配置 `TAURI_SIGNING_PRIVATE_KEY`（私钥文件内容）
   和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（密码文件内容）
3. 发版：`git tag v0.2.0 && git push origin v0.2.0`，Release 工作流自动构建
   dmg / app.tar.gz / latest.json 并发布
4. 已安装应用在「软件更新」中检查 → 下载 → 安装 → 重启，全程无需手动重装

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
│   │   ├── ai_chat.rs     # AI 对话插件（流式请求/对话持久化）
│   │   ├── translator.rs   # 聚合翻译插件（MyMemory/Google 引擎）
│   │   ├── calc.rs         # 计算稿纸插件（表达式求值器）
│   │   └── main.rs        # 入口文件
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
└── package.json           # Node 依赖
```

## License

MIT
