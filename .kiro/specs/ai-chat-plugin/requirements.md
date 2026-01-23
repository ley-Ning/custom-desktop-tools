# Requirements Document

## Introduction

AI 对话插件是 My uTools 应用的一个内置插件，允许用户通过自然语言与配置的 AI 模型进行对话。该插件利用已有的 AI 模型配置系统，提供流畅的对话体验，支持多轮对话、历史记录管理和上下文保持。

## Glossary

- **AI Chat Plugin**: AI 对话插件，My uTools 的内置插件，用于与 AI 模型进行对话
- **Conversation**: 对话，一次完整的用户与 AI 的交互会话
- **Message**: 消息，对话中的单条用户输入或 AI 回复
- **Context**: 上下文，对话历史中保留的消息，用于维持对话连贯性
- **AI Model**: AI 模型，在 AI 模型配置中定义的可用模型
- **Default Model**: 默认模型，用户设置为默认的 AI 模型
- **Streaming Response**: 流式响应，AI 逐字返回回复内容的方式
- **Token**: 令牌，AI 模型处理文本的基本单位
- **System Prompt**: 系统提示词，定义 AI 行为和角色的初始指令

## Requirements

### Requirement 1

**User Story:** 作为用户，我想要通过关键词快速打开 AI 对话插件，以便快速开始与 AI 的对话。

#### Acceptance Criteria

1. WHEN 用户在搜索框输入"ai"、"chat"、"对话"或"聊天" THEN AI Chat Plugin SHALL 出现在插件推荐列表中
2. WHEN 用户选择 AI 对话插件 THEN AI Chat Plugin SHALL 显示对话界面
3. WHEN 用户按下 Esc 键 THEN AI Chat Plugin SHALL 关闭并返回主界面
4. WHEN AI 对话插件打开时 THEN AI Chat Plugin SHALL 自动聚焦到输入框

### Requirement 2

**User Story:** 作为用户，我想要选择不同的 AI 模型进行对话，以便根据任务选择最合适的模型。

#### Acceptance Criteria

1. WHEN AI 对话插件打开时 THEN AI Chat Plugin SHALL 显示当前选中的模型名称
2. WHEN 用户点击模型选择器 THEN AI Chat Plugin SHALL 显示所有已启用的 AI 模型列表
3. WHEN 用户选择一个模型 THEN AI Chat Plugin SHALL 切换到该模型并更新界面显示
4. WHEN 没有可用的 AI 模型时 THEN AI Chat Plugin SHALL 显示提示信息引导用户配置模型
5. WHEN AI 对话插件首次打开时 THEN AI Chat Plugin SHALL 自动选择默认模型

### Requirement 3

**User Story:** 作为用户，我想要发送消息给 AI 并接收回复，以便与 AI 进行对话。

#### Acceptance Criteria

1. WHEN 用户在输入框输入消息并按下 Enter 键 THEN AI Chat Plugin SHALL 发送消息到 AI 模型
2. WHEN 用户在输入框输入消息并按下 Shift+Enter THEN AI Chat Plugin SHALL 在输入框中插入换行符
3. WHEN 消息正在发送时 THEN AI Chat Plugin SHALL 禁用输入框和发送按钮
4. WHEN AI 返回响应时 THEN AI Chat Plugin SHALL 以流式方式显示回复内容
5. WHEN 用户输入空消息时 THEN AI Chat Plugin SHALL 阻止发送并保持当前状态
6. WHEN 消息发送失败时 THEN AI Chat Plugin SHALL 显示错误信息并允许重试

### Requirement 4

**User Story:** 作为用户，我想要查看完整的对话历史，以便回顾之前的对话内容。

#### Acceptance Criteria

1. WHEN 新消息添加到对话中 THEN AI Chat Plugin SHALL 将消息追加到对话历史末尾
2. WHEN 对话历史更新时 THEN AI Chat Plugin SHALL 自动滚动到最新消息
3. WHEN 用户滚动查看历史消息时 THEN AI Chat Plugin SHALL 保持滚动位置不自动跳转
4. WHEN 对话历史为空时 THEN AI Chat Plugin SHALL 显示欢迎消息和使用提示
5. WHEN 对话历史包含代码块时 THEN AI Chat Plugin SHALL 正确渲染代码高亮

### Requirement 5

**User Story:** 作为用户，我想要管理多个对话会话，以便在不同主题之间切换。

#### Acceptance Criteria

1. WHEN 用户点击"新建对话"按钮 THEN AI Chat Plugin SHALL 创建新的空白对话并切换到该对话
2. WHEN 用户在对话列表中选择一个对话 THEN AI Chat Plugin SHALL 加载该对话的历史记录
3. WHEN 用户删除一个对话时 THEN AI Chat Plugin SHALL 显示确认对话框
4. WHEN 用户确认删除对话 THEN AI Chat Plugin SHALL 删除该对话并切换到最近的对话
5. WHEN 对话列表为空时 THEN AI Chat Plugin SHALL 自动创建一个新对话

### Requirement 6

**User Story:** 作为用户，我想要对话数据能够持久化保存，以便下次打开时继续之前的对话。

#### Acceptance Criteria

1. WHEN 用户发送或接收消息时 THEN AI Chat Plugin SHALL 立即将对话数据保存到本地存储
2. WHEN AI 对话插件打开时 THEN AI Chat Plugin SHALL 从本地存储加载所有对话历史
3. WHEN 对话数据保存失败时 THEN AI Chat Plugin SHALL 显示错误提示但不影响当前对话
4. WHEN 对话数据损坏时 THEN AI Chat Plugin SHALL 创建新的空白对话并记录错误
5. WHEN 对话历史超过 100 条对话时 THEN AI Chat Plugin SHALL 保留最近的 100 条对话

### Requirement 7

**User Story:** 作为用户，我想要复制 AI 的回复内容，以便在其他地方使用。

#### Acceptance Criteria

1. WHEN 用户点击消息的复制按钮 THEN AI Chat Plugin SHALL 将消息内容复制到系统剪贴板
2. WHEN 复制成功时 THEN AI Chat Plugin SHALL 显示"已复制"提示
3. WHEN 复制失败时 THEN AI Chat Plugin SHALL 显示错误提示
4. WHEN 消息包含代码块时 THEN AI Chat Plugin SHALL 复制纯文本内容包括代码

### Requirement 8

**User Story:** 作为用户，我想要停止 AI 正在生成的回复，以便节省时间和 API 调用成本。

#### Acceptance Criteria

1. WHEN AI 正在生成回复时 THEN AI Chat Plugin SHALL 显示"停止生成"按钮
2. WHEN 用户点击"停止生成"按钮 THEN AI Chat Plugin SHALL 中断 API 请求
3. WHEN 生成被停止时 THEN AI Chat Plugin SHALL 保留已生成的部分内容
4. WHEN 生成被停止时 THEN AI Chat Plugin SHALL 重新启用输入框和发送按钮

### Requirement 9

**User Story:** 作为用户，我想要清空当前对话，以便开始全新的对话主题。

#### Acceptance Criteria

1. WHEN 用户点击"清空对话"按钮 THEN AI Chat Plugin SHALL 显示确认对话框
2. WHEN 用户确认清空对话 THEN AI Chat Plugin SHALL 删除当前对话的所有消息
3. WHEN 对话被清空后 THEN AI Chat Plugin SHALL 显示欢迎消息
4. WHEN 对话被清空后 THEN AI Chat Plugin SHALL 保持当前选中的模型不变

### Requirement 10

**User Story:** 作为用户，我想要设置系统提示词，以便自定义 AI 的行为和角色。

#### Acceptance Criteria

1. WHEN 用户点击"设置"按钮 THEN AI Chat Plugin SHALL 显示设置面板
2. WHEN 用户在设置面板输入系统提示词 THEN AI Chat Plugin SHALL 保存系统提示词到对话配置
3. WHEN 系统提示词被设置时 THEN AI Chat Plugin SHALL 在发送消息时包含系统提示词
4. WHEN 用户清空系统提示词 THEN AI Chat Plugin SHALL 使用默认的系统提示词
5. WHEN 切换对话时 THEN AI Chat Plugin SHALL 加载该对话的系统提示词设置

### Requirement 11

**User Story:** 作为用户，我想要调整对话的上下文长度，以便控制 AI 记忆的对话历史数量。

#### Acceptance Criteria

1. WHEN 用户在设置面板调整上下文长度滑块 THEN AI Chat Plugin SHALL 更新上下文长度值
2. WHEN 发送消息时 THEN AI Chat Plugin SHALL 只包含最近 N 条消息作为上下文
3. WHEN 上下文长度设置为 0 时 THEN AI Chat Plugin SHALL 不包含历史消息只发送当前消息
4. WHEN 上下文长度大于实际消息数量时 THEN AI Chat Plugin SHALL 包含所有历史消息

### Requirement 12

**User Story:** 作为用户，我想要看到 API 调用的统计信息，以便了解使用情况和成本。

#### Acceptance Criteria

1. WHEN 对话界面显示时 THEN AI Chat Plugin SHALL 显示当前对话的消息数量
2. WHEN 对话界面显示时 THEN AI Chat Plugin SHALL 显示当前对话的大约 token 使用量
3. WHEN API 调用完成时 THEN AI Chat Plugin SHALL 更新统计信息
4. WHEN 切换对话时 THEN AI Chat Plugin SHALL 显示对应对话的统计信息

### Requirement 13

**User Story:** 作为开发者，我想要 AI 对话插件与现有的插件系统集成，以便保持架构一致性。

#### Acceptance Criteria

1. WHEN AI 对话插件被触发时 THEN AI Chat Plugin SHALL 通过插件匹配系统被识别
2. WHEN AI 对话插件关闭时 THEN AI Chat Plugin SHALL 触发 @close 事件返回主界面
3. WHEN AI 对话插件使用 AI 模型时 THEN AI Chat Plugin SHALL 从 AI 模型配置中读取模型信息
4. WHEN AI 对话插件保存数据时 THEN AI Chat Plugin SHALL 使用 tauri-plugin-store 保存到 ai-chat.json

### Requirement 14

**User Story:** 作为用户，我想要对话界面响应迅速且流畅，以便获得良好的使用体验。

#### Acceptance Criteria

1. WHEN 用户输入消息时 THEN AI Chat Plugin SHALL 在 100ms 内响应键盘输入
2. WHEN AI 返回流式响应时 THEN AI Chat Plugin SHALL 在 50ms 内更新显示内容
3. WHEN 对话历史包含超过 50 条消息时 THEN AI Chat Plugin SHALL 使用虚拟滚动优化性能
4. WHEN 切换对话时 THEN AI Chat Plugin SHALL 在 200ms 内完成加载和渲染

### Requirement 15

**User Story:** 作为用户，我想要在 AI 回复中看到格式化的内容，以便更好地阅读和理解。

#### Acceptance Criteria

1. WHEN AI 回复包含 Markdown 格式时 THEN AI Chat Plugin SHALL 正确渲染 Markdown 内容
2. WHEN AI 回复包含代码块时 THEN AI Chat Plugin SHALL 显示语法高亮的代码
3. WHEN AI 回复包含列表时 THEN AI Chat Plugin SHALL 正确渲染有序和无序列表
4. WHEN AI 回复包含链接时 THEN AI Chat Plugin SHALL 渲染可点击的链接
5. WHEN AI 回复包含表格时 THEN AI Chat Plugin SHALL 正确渲染表格格式
