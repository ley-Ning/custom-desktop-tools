# Implementation Tasks — 聚合翻译插件

> 状态：✅ 已完成（2026-09-16）
> 验证：`cargo test --lib` 18 个测试通过；`vue-tsc --noEmit` 0 错误；`vite build` 成功；`cargo check --all-targets` 通过。

## 需求概要

- 关键词「翻译 / translate / fanyi / fy / 中英文等」唤出插件；搜索框中的文本自动带入并立即翻译（uTools 式输入即翻译）
- 多引擎聚合：MyMemory（默认，免 key、支持 Autodetect、返回检测语言）、Google（gtx 端点，部分地区不可达）、AI 翻译（复用 ai-config 已启用模型，流式输出）
- 源语言自动检测 + 智能目标语言（中文↔英文自动判断方向）、手动选择 12 种语言、一键交换（译文填回输入框）
- 输入防抖 600ms 自动翻译；Enter 立即翻译，Shift+Enter 换行；粘贴后快速翻译
- 复制译文 + 反馈；错误内联展示；MyMemory 500 字节上限提示与拦截

## 实现清单

### 后端（src-tauri/src/translator.rs）

- [x] `translate_text(text, from, to, engine)` 命令：参数校验（空文本 / 目标为 auto / 未知引擎）
- [x] MyMemory 引擎：`langpair=Autodetect|<to>` 自动检测，解析 `responseData.translatedText` + `detectedLanguage`；`responseStatus != 200` 与 `MYMEMORY WARNING`（免费额度耗尽）转错误；500 字节上限前置校验
- [x] Google 引擎：`translate_a/single` gtx 端点，多段译文拼接 + `sl=auto` 检测语言解析
- [x] 9 个单元测试：gtx 单段/多段/非法响应、MyMemory 正常/无检测语言/额度警告/错误状态码、长度上限、参数校验

### 前端（src/components/TranslatorPlugin.vue）

- [x] 双卡片布局（源文/译文）+ 语言选择器 + 交换按钮（自动检测时用检测语言回填）
- [x] 引擎选择器；配置过 AI 模型时才出现「AI 翻译」选项（附模型下拉）
- [x] AI 翻译复用 `send_ai_message` 流式通道（message_id 前缀 `translator-` 过滤事件），流式光标、停止/错误收尾
- [x] 智能目标语言（CJK 检测 → en，否则 → zh-CN），仅目标未被手动修改时跟随
- [x] 自动翻译（防抖 600ms）/ Enter 立即翻译 / 粘贴快速翻译 / 切换引擎自动重译
- [x] 字节计数（UTF-8）与 MyMemory 超限提示、复制反馈、空态引导

### 集成

- [x] App.vue：插件入口（搜索匹配 / 智能推荐首选 / Esc 关闭 / 状态持久化 / URL 参数），搜索文本经 `initial-text` prop 带入
- [x] db/index.ts：默认启用，补充「日语 / riyu」关键词

## 已知限制

- Google 引擎在部分地区（含本机网络）不可达，错误信息会提示切换引擎
- MyMemory 匿名额度约 5000 字符/天，超限提示改用 AI 翻译
- AI 翻译与 AI 对话共享单一生成槽位（同一时间只能有一个流式请求）
