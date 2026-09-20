# Implementation Tasks

> 状态：✅ 已全部完成（2026-09-16）
> 验证：`cargo test --lib` 9 个测试通过；`vue-tsc --noEmit` 0 错误；`vite build` 成功；`cargo check --all-targets` 通过。

## 1. 后端（src-tauri/src/ai_chat.rs）

- [x] 数据结构：`ChatMessageRecord` / `Conversation` / `ConversationSettings` / `ApiChatMessage` / `AiModelConfig`（与前端 types/index.ts 一一对应，snake_case）
- [x] `send_ai_message` 命令：加载 ai-config.json → 校验模型已启用 → OpenAI 兼容 `/chat/completions` 流式请求 → `ai-message-chunk` 事件逐段推送 → `ai-message-done` 事件收尾（tokio::spawn 后台任务，命令立即返回）
- [x] SSE 解析：跨 chunk 缓冲按行切分，`[DONE]` 处理，容错忽略 keep-alive/不可解析行
- [x] `stop_ai_generation` 命令：JoinHandle.abort() 中断请求并补发 `done(stopped)` 事件（保留已生成内容由前端负责）
- [x] `load_conversations` / `save_conversation` / `delete_conversation`：tauri-plugin-store 持久化到 ai-chat.json，超过 100 条按 updated_at 淘汰最旧
- [x] `open_external_url` 命令：Markdown 链接经系统浏览器打开（仅 http/https）
- [x] 单元测试：请求体序列化、SSE chunk 解析（内容/空 delta/[DONE]/非法行）、对话数量限制、serde 往返、旧数据缺省 settings 兼容

## 2. 前端

- [x] `src/types/index.ts`：新增 `AiModel` / `AiChatMessage` / `Conversation` / `ConversationSettings` / `ApiChatMessage` / `StreamChunkEvent` / `StreamDoneEvent`
- [x] `src/components/AiChatPlugin.vue`：完整对话界面
  - 多会话管理：新建/切换/删除（确认框）/清空（确认框），自动标题，列表为空自动新建
  - 模型选择：仅列出已启用模型，自动选择默认模型，未配置时显示引导
  - 消息收发：Enter 发送 / Shift+Enter 换行 / 空消息阻止 / 生成中禁用输入与发送
  - 流式渲染：chunk 事件追加 + 闪烁光标；停止生成保留部分内容；失败显示错误 + 重试
  - Markdown：marked + highlight.js 代码高亮 + DOMPurify 消毒，渲染缓存（流式时仅重算变化消息）
  - 设置面板：系统提示词（每对话独立）、上下文长度滑块（0-50，0 表示不携带历史）
  - 统计：消息数 + token 估算（CJK 与非 CJK 分别估算）
  - 复制消息到系统剪贴板 + “已复制”反馈
  - 滚动：靠近底部才自动跟随，上滑查看历史不跳转
  - 性能：消息项 `content-visibility: auto`（超过 50 条时跳过屏外渲染）
- [x] `AiModelSettings.vue`：复用 `types/index.ts` 的 `AiModel`，消除重复定义

## 3. 集成与接线

- [x] `App.vue`：注册 AI 对话插件入口（搜索关键词匹配 / 快捷插件格子 / Esc 关闭 / 状态持久化 / URL 参数）
- [x] `db/index.ts`：AI 对话插件默认启用，补充 `liaotian` 关键词

## 4. 顺带修复的存量构建阻塞（main 分支 `pnpm build` 本就无法通过）

- [x] `db/index.ts`：补齐 `addClipboardItem` / `deleteClipboardItem` / `clearClipboardHistory` / `toggleClipboardFavorite` / `addMemo` / `updateMemo` / `deleteMemo` / `toggleMemoPinned`（改为 Rust 命令封装，含 snake_case → camelCase、秒 → 毫秒映射，修复剪贴板/备忘录插件运行时崩溃）
- [x] `db/index.ts`：`getEnabledPlugins` 改为 getAll + 过滤（原 `getAllFromIndex('by-enabled', true)` 因 boolean 不是合法 IDB key 运行时抛 DataError，导致插件列表加载失败）
- [x] `JsonEditorPlugin.vue`：迁移到 vanilla-jsoneditor v3.11 API（`createJSONEditor` + `JsonEditor` 类型 + `isContentValidationErrors` / `isJSONContent` / `isTextContent` 收窄）
- [x] `TimestampPlugin.vue`：补 dayjs 插件（quarterOfYear / weekOfYear / dayOfYear），copyToClipboard 参数收窄
- [x] `MemoPlugin.vue`：`updatedAt` → `updated_at` 字段名对齐
- [x] `PluginList.vue` / `App.vue`：清理未使用变量
- [x] `pnpm-workspace.yaml`：清除无效的 `allowBuilds` 占位内容，改为 `onlyBuiltDependencies`

## 5. 未实现（超出本期范围，见 design.md Phase 2）

- 虚拟滚动组件级实现（以 `content-visibility` 等效满足 Requirement 14.3）
- 多模态输入、对话导出、函数调用

## 6. 后续增强（2026-09-16 第二轮优化）

- [x] ApiProtocol 抽象：按模型 provider 字段自动选择协议（anthropic → 原生 /v1/messages，其余 → OpenAI 兼容）
- [x] Anthropic 原生协议支持：`x-api-key` + `anthropic-version` 认证头、system 独立顶层字段、必填 max_tokens（默认 4096）、`content_block_delta`/`message_stop`/`error` SSE 事件解析（thinking 增量不混入正文）
- [x] 新增 6 个单元测试（协议选择、请求体序列化、SSE 事件族），全套 33 个测试通过
