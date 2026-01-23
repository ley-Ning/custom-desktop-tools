# Design Document

## Overview

AI 对话插件是 My uTools 应用的一个核心插件，提供与 AI 模型的自然语言交互能力。该插件采用现代化的对话界面设计，支持流式响应、多会话管理、Markdown 渲染等功能。插件完全集成到现有的插件系统架构中，复用 AI 模型配置系统，提供一致的用户体验。

## Architecture

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      My uTools 主应用                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐      ┌──────────────┐                    │
│  │  搜索系统    │─────▶│ 插件匹配器   │                    │
│  └──────────────┘      └──────┬───────┘                    │
│                               │                             │
│                               ▼                             │
│                    ┌──────────────────┐                    │
│                    │  AI Chat Plugin  │                    │
│                    └────────┬─────────┘                    │
│                             │                               │
│         ┌───────────────────┼───────────────────┐          │
│         │                   │                   │          │
│         ▼                   ▼                   ▼          │
│  ┌─────────────┐   ┌──────────────┐   ┌──────────────┐   │
│  │ 对话管理器  │   │  消息渲染器  │   │  设置管理器  │   │
│  └──────┬──────┘   └──────────────┘   └──────────────┘   │
│         │                                                   │
│         ▼                                                   │
│  ┌─────────────┐                                           │
│  │ AI 模型配置 │                                           │
│  └──────┬──────┘                                           │
│         │                                                   │
└─────────┼───────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────┐
│                      Tauri Backend (Rust)                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────┐   │
│  │ AI 配置管理  │   │ 对话数据存储 │   │  HTTP 客户端 │   │
│  └──────────────┘   └──────────────┘   └──────┬───────┘   │
│                                                 │           │
└─────────────────────────────────────────────────┼───────────┘
                                                  │
                                                  ▼
                                        ┌──────────────────┐
                                        │  AI API 服务商   │
                                        │ (OpenAI/Claude)  │
                                        └──────────────────┘
```

### Component Interaction Flow

```
用户输入消息
    │
    ▼
AI Chat Plugin (Vue)
    │
    ├─▶ 验证输入
    │
    ├─▶ 构建请求上下文
    │   ├─ 加载对话历史
    │   ├─ 应用系统提示词
    │   └─ 限制上下文长度
    │
    ├─▶ 调用 Rust 后端
    │   └─ send_ai_message(model_id, messages)
    │
    ▼
Tauri Backend (Rust)
    │
    ├─▶ 加载 AI 模型配置
    │
    ├─▶ 构建 HTTP 请求
    │   ├─ 设置 Headers (Authorization, Content-Type)
    │   ├─ 构建请求体 (messages, model, stream: true)
    │   └─ 设置超时
    │
    ├─▶ 发送 HTTP 请求到 AI API
    │
    ├─▶ 处理流式响应
    │   ├─ 解析 SSE (Server-Sent Events)
    │   ├─ 提取 delta 内容
    │   └─ 发送事件到前端
    │
    └─▶ 错误处理
        ├─ 网络错误
        ├─ API 错误
        └─ 超时错误
    │
    ▼
AI Chat Plugin (Vue)
    │
    ├─▶ 接收流式响应事件
    │   └─ ai-message-chunk
    │
    ├─▶ 更新 UI
    │   ├─ 追加文本到消息
    │   ├─ 渲染 Markdown
    │   └─ 自动滚动
    │
    ├─▶ 保存对话历史
    │   └─ save_conversation(conversation)
    │
    └─▶ 更新统计信息
```

## Components and Interfaces

### Frontend Components

#### 1. AiChatPlugin.vue (主组件)

**职责：**
- 插件生命周期管理
- 对话列表管理
- 模型选择
- 全局状态管理

**Props：** 无

**Emits：**
- `close`: 关闭插件

**主要方法：**
```typescript
// 对话管理
loadConversations(): Promise<void>
createNewConversation(): void
selectConversation(id: string): void
deleteConversation(id: string): Promise<void>

// 模型管理
loadAvailableModels(): Promise<void>
selectModel(modelId: string): void
```

#### 2. ChatConversation.vue (对话视图)

**职责：**
- 消息列表渲染
- 消息发送
- 流式响应处理
- 滚动管理

**Props：**
```typescript
interface Props {
  conversation: Conversation;
  selectedModel: AiModel;
}
```

**Emits：**
- `update:conversation`: 对话更新

**主要方法：**
```typescript
// 消息处理
sendMessage(content: string): Promise<void>
stopGeneration(): void
retryMessage(messageId: string): Promise<void>

// UI 交互
scrollToBottom(): void
copyMessage(content: string): void
```

#### 3. MessageItem.vue (消息项)

**职责：**
- 单条消息渲染
- Markdown 解析
- 代码高亮
- 消息操作

**Props：**
```typescript
interface Props {
  message: Message;
  isStreaming: boolean;
}
```

**Emits：**
- `copy`: 复制消息
- `retry`: 重试消息

#### 4. ChatSettings.vue (设置面板)

**职责：**
- 系统提示词配置
- 上下文长度设置
- 温度参数设置

**Props：**
```typescript
interface Props {
  conversation: Conversation;
}
```

**Emits：**
- `update:settings`: 设置更新

#### 5. ConversationList.vue (对话列表)

**职责：**
- 对话列表显示
- 对话切换
- 对话删除

**Props：**
```typescript
interface Props {
  conversations: Conversation[];
  currentId: string;
}
```

**Emits：**
- `select`: 选择对话
- `delete`: 删除对话
- `new`: 新建对话

### Backend Commands (Rust)

#### 1. AI 消息发送

```rust
#[command]
async fn send_ai_message(
    model_id: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    window: Window,
) -> Result<String, String>
```

**功能：**
- 加载指定的 AI 模型配置
- 构建 HTTP 请求
- 处理流式或非流式响应
- 通过事件发送响应块到前端

**返回：**
- 成功：完整的 AI 回复文本
- 失败：错误信息

#### 2. 停止生成

```rust
#[command]
async fn stop_ai_generation() -> Result<(), String>
```

**功能：**
- 中断当前的 AI 请求
- 清理资源

#### 3. 对话数据管理

```rust
#[command]
async fn load_conversations() -> Result<Vec<Conversation>, String>

#[command]
async fn save_conversation(conversation: Conversation) -> Result<(), String>

#[command]
async fn delete_conversation(id: String) -> Result<(), String>
```

**功能：**
- 从本地存储加载对话列表
- 保存对话到本地存储
- 删除指定对话

#### 4. 获取可用模型

```rust
#[command]
async fn get_available_ai_models() -> Result<Vec<AiModel>, String>
```

**功能：**
- 从 AI 配置中读取所有已启用的模型
- 返回模型列表供前端选择

## Data Models

### TypeScript Interfaces

```typescript
// 消息类型
interface Message {
  id: string;                    // UUID
  role: 'user' | 'assistant' | 'system';
  content: string;               // 消息内容
  timestamp: number;             // Unix 时间戳
  tokens?: number;               // Token 数量（估算）
  error?: string;                // 错误信息
}

// 对话类型
interface Conversation {
  id: string;                    // UUID
  title: string;                 // 对话标题（自动生成或用户设置）
  messages: Message[];           // 消息列表
  model_id: string;              // 使用的模型 ID
  created_at: number;            // 创建时间
  updated_at: number;            // 最后更新时间
  settings: ConversationSettings; // 对话设置
}

// 对话设置
interface ConversationSettings {
  system_prompt: string;         // 系统提示词
  context_length: number;        // 上下文长度（消息数量）
  temperature?: number;          // 温度参数 (0-2)
  max_tokens?: number;           // 最大生成 token 数
}

// AI 模型（复用现有类型）
interface AiModel {
  id: string;
  name: string;
  provider: string;
  baseUrl: string;
  apiKey: string;
  model: string;
  enabled: boolean;
  isDefault: boolean;
}

// 聊天消息（API 请求格式）
interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

// 流式响应事件
interface StreamChunkEvent {
  conversation_id: string;
  message_id: string;
  content: string;              // Delta 内容
  done: boolean;                // 是否完成
}
```

### Rust Structs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    id: String,
    role: String,  // "user", "assistant", "system"
    content: String,
    timestamp: i64,
    tokens: Option<i32>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Conversation {
    id: String,
    title: String,
    messages: Vec<Message>,
    model_id: String,
    created_at: i64,
    updated_at: i64,
    settings: ConversationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConversationSettings {
    system_prompt: String,
    context_length: i32,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AiModel {
    id: String,
    name: String,
    provider: String,
    base_url: String,
    api_key: String,
    model: String,
    enabled: bool,
    is_default: bool,
}

// OpenAI API 请求格式
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    temperature: Option<f32>,
    max_tokens: Option<i32>,
}

// OpenAI API 响应格式（流式）
#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    index: i32,
    delta: Delta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    role: Option<String>,
    content: Option<String>,
}
```

## 
Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Plugin keyword matching

*For any* search query containing "ai", "chat", "对话", or "聊天", the plugin matching system should return AI Chat Plugin in the results
**Validates: Requirements 1.1**

### Property 2: Model list filtering

*For any* AI model configuration, only models with enabled=true should appear in the model selection list
**Validates: Requirements 2.2**

### Property 3: Model selection updates state

*For any* valid AI model, selecting it should update the current model state and UI display
**Validates: Requirements 2.3**

### Property 4: Default model auto-selection

*For any* AI configuration with a default model, opening the plugin should automatically select that model
**Validates: Requirements 2.5**

### Property 5: Non-empty message sending

*For any* non-empty message string, pressing Enter should trigger the send message action
**Validates: Requirements 3.1**

### Property 6: Empty message rejection

*For any* string composed entirely of whitespace, attempting to send should be blocked and state should remain unchanged
**Validates: Requirements 3.5**

### Property 7: Message appending order

*For any* new message, it should be appended to the end of the conversation messages array
**Validates: Requirements 4.1**

### Property 8: Message list persistence round-trip

*For any* conversation with messages, saving then loading should produce an equivalent conversation
**Validates: Requirements 6.1, 6.2**

### Property 9: Conversation limit enforcement

*For any* conversation list exceeding 100 items, only the most recent 100 conversations should be retained
**Validates: Requirements 6.5**

### Property 10: Message copy to clipboard

*For any* message content, clicking copy should write that content to the system clipboard
**Validates: Requirements 7.1**

### Property 11: Code block copy includes text

*For any* message containing code blocks, copying should include the code as plain text
**Validates: Requirements 7.4**

### Property 12: Stop generation preserves partial content

*For any* streaming response that is stopped, the already-generated content should be preserved in the message
**Validates: Requirements 8.3**

### Property 13: Stop generation re-enables UI

*For any* stopped generation, the input box and send button should be re-enabled
**Validates: Requirements 8.4**

### Property 14: Clear conversation removes all messages

*For any* conversation, confirming clear should result in an empty messages array
**Validates: Requirements 9.2**

### Property 15: Clear conversation preserves model

*For any* conversation with a selected model, clearing the conversation should not change the selected model
**Validates: Requirements 9.4**

### Property 16: System prompt persistence

*For any* system prompt text, saving it should persist to the conversation settings
**Validates: Requirements 10.2**

### Property 17: System prompt inclusion in requests

*For any* conversation with a system prompt, sending a message should include the system prompt in the API request
**Validates: Requirements 10.3**

### Property 18: System prompt isolation per conversation

*For any* two different conversations, each should maintain its own independent system prompt setting
**Validates: Requirements 10.5**

### Property 19: Context length limiting

*For any* context length setting N and message history, only the most recent N messages should be included in API requests
**Validates: Requirements 11.2**

### Property 20: Message count accuracy

*For any* conversation, the displayed message count should equal the length of the messages array
**Validates: Requirements 12.1**

### Property 21: Statistics update after API call

*For any* completed API call, the conversation statistics should be updated
**Validates: Requirements 12.3**

### Property 22: Statistics isolation per conversation

*For any* two different conversations, each should display its own independent statistics
**Validates: Requirements 12.4**

### Property 23: Model configuration integration

*For any* enabled AI model in the configuration, the chat plugin should be able to load and use that model's settings
**Validates: Requirements 13.3**

### Property 24: Data storage persistence

*For any* conversation data saved, it should be stored in ai-chat.json using tauri-plugin-store
**Validates: Requirements 13.4**

### Property 25: Virtual scrolling for large histories

*For any* conversation with more than 50 messages, the message list should use virtual scrolling
**Validates: Requirements 14.3**

### Property 26: Markdown rendering correctness

*For any* valid Markdown content, the renderer should produce correctly formatted HTML
**Validates: Requirements 15.1**

### Property 27: Code block syntax highlighting

*For any* code block with a language specifier, syntax highlighting should be applied
**Validates: Requirements 15.2**

### Property 28: List rendering correctness

*For any* Markdown list (ordered or unordered), the renderer should produce the correct list structure
**Validates: Requirements 15.3**

### Property 29: Link rendering as clickable

*For any* Markdown link, the renderer should produce a clickable anchor element
**Validates: Requirements 15.4**

### Property 30: Table rendering correctness

*For any* Markdown table, the renderer should produce a correctly structured HTML table
**Validates: Requirements 15.5**

## Error Handling

### Frontend Error Handling

#### 1. Network Errors

**Scenarios:**
- API 请求超时
- 网络连接断开
- DNS 解析失败

**Handling:**
```typescript
try {
  await sendMessage(content);
} catch (error) {
  if (error.type === 'NetworkError') {
    showError('网络连接失败，请检查网络设置');
    enableRetry();
  }
}
```

#### 2. API Errors

**Scenarios:**
- 401 Unauthorized (API Key 无效)
- 429 Too Many Requests (速率限制)
- 500 Internal Server Error (服务器错误)

**Handling:**
```typescript
if (error.status === 401) {
  showError('API Key 无效，请检查配置');
  openSettings();
} else if (error.status === 429) {
  showError('请求过于频繁，请稍后再试');
  disableSendFor(60); // 禁用 60 秒
} else if (error.status >= 500) {
  showError('AI 服务暂时不可用，请稍后再试');
  enableRetry();
}
```

#### 3. Data Errors

**Scenarios:**
- 对话数据损坏
- JSON 解析失败
- 存储空间不足

**Handling:**
```typescript
try {
  const conversations = await loadConversations();
} catch (error) {
  console.error('Failed to load conversations:', error);
  // 创建新的空白对话
  createDefaultConversation();
  showWarning('对话历史加载失败，已创建新对话');
}
```

#### 4. Validation Errors

**Scenarios:**
- 空消息
- 消息过长
- 无效的模型配置

**Handling:**
```typescript
function validateMessage(content: string): boolean {
  if (!content.trim()) {
    showError('消息不能为空');
    return false;
  }
  if (content.length > 10000) {
    showError('消息过长，请缩短后重试');
    return false;
  }
  return true;
}
```

### Backend Error Handling

#### 1. HTTP Client Errors

```rust
async fn send_ai_message(
    model_id: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    window: Window,
) -> Result<String, String> {
    // 加载模型配置
    let model = load_model_config(&model_id)
        .map_err(|e| format!("Failed to load model config: {}", e))?;
    
    // 构建请求
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // 发送请求
    let response = client
        .post(&format!("{}/chat/completions", model.base_url))
        .header("Authorization", format!("Bearer {}", model.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    // 检查状态码
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, error_text));
    }
    
    // 处理响应...
    Ok(full_response)
}
```

#### 2. Stream Processing Errors

```rust
// 处理流式响应
let mut stream = response.bytes_stream();
let mut buffer = String::new();

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes);
            // 解析 SSE 格式
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        break;
                    }
                    
                    match serde_json::from_str::<ChatCompletionChunk>(data) {
                        Ok(chunk) => {
                            // 发送事件到前端
                            window.emit("ai-message-chunk", chunk).ok();
                        }
                        Err(e) => {
                            eprintln!("Failed to parse chunk: {}", e);
                            continue;
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Stream error: {}", e));
        }
    }
}
```

#### 3. Data Storage Errors

```rust
async fn save_conversation(conversation: Conversation) -> Result<(), String> {
    let store = get_store()
        .map_err(|e| format!("Failed to get store: {}", e))?;
    
    // 加载现有对话列表
    let mut conversations: Vec<Conversation> = store
        .get("conversations")
        .unwrap_or_default();
    
    // 更新或添加对话
    if let Some(index) = conversations.iter().position(|c| c.id == conversation.id) {
        conversations[index] = conversation;
    } else {
        conversations.push(conversation);
    }
    
    // 限制对话数量
    if conversations.len() > 100 {
        conversations = conversations
            .into_iter()
            .rev()
            .take(100)
            .rev()
            .collect();
    }
    
    // 保存
    store
        .set("conversations", serde_json::to_value(&conversations).unwrap())
        .map_err(|e| format!("Failed to save conversations: {}", e))?;
    
    store
        .save()
        .map_err(|e| format!("Failed to persist store: {}", e))?;
    
    Ok(())
}
```

## Testing Strategy

### Unit Testing

#### Frontend Unit Tests (Vitest)

**测试范围：**
- 组件渲染
- 用户交互
- 状态管理
- 数据验证

**示例测试：**

```typescript
// tests/AiChatPlugin.test.ts
import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import AiChatPlugin from '@/components/AiChatPlugin.vue';

describe('AiChatPlugin', () => {
  it('should display empty state when no conversations', () => {
    const wrapper = mount(AiChatPlugin, {
      props: {},
    });
    
    expect(wrapper.find('.empty-state').exists()).toBe(true);
    expect(wrapper.text()).toContain('开始新对话');
  });
  
  it('should create new conversation on button click', async () => {
    const wrapper = mount(AiChatPlugin);
    
    await wrapper.find('.btn-new-conversation').trigger('click');
    
    expect(wrapper.vm.conversations.length).toBe(1);
    expect(wrapper.vm.currentConversation).toBeTruthy();
  });
  
  it('should reject empty messages', async () => {
    const wrapper = mount(AiChatPlugin);
    const sendMessage = vi.spyOn(wrapper.vm, 'sendMessage');
    
    wrapper.vm.messageInput = '   ';
    await wrapper.find('.btn-send').trigger('click');
    
    expect(sendMessage).not.toHaveBeenCalled();
  });
});
```

#### Backend Unit Tests (Rust)

**测试范围：**
- API 请求构建
- 响应解析
- 数据序列化
- 错误处理

**示例测试：**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_chat_request() {
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
        ];
        
        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages,
            stream: true,
            temperature: Some(0.7),
            max_tokens: None,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"gpt-4\""));
        assert!(json.contains("\"stream\":true"));
    }
    
    #[test]
    fn test_parse_stream_chunk() {
        let json = r#"{"id":"123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        
        let chunk: ChatCompletionChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.id, "123");
        assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
    }
    
    #[tokio::test]
    async fn test_conversation_limit() {
        let mut conversations = Vec::new();
        for i in 0..150 {
            conversations.push(Conversation {
                id: format!("conv-{}", i),
                title: format!("Conversation {}", i),
                messages: vec![],
                model_id: "model-1".to_string(),
                created_at: i,
                updated_at: i,
                settings: ConversationSettings::default(),
            });
        }
        
        // 应用限制
        conversations = conversations
            .into_iter()
            .rev()
            .take(100)
            .rev()
            .collect();
        
        assert_eq!(conversations.len(), 100);
        assert_eq!(conversations[0].id, "conv-50");
        assert_eq!(conversations[99].id, "conv-149");
    }
}
```

### Property-Based Testing

我们将使用 **fast-check** (TypeScript) 和 **proptest** (Rust) 进行属性测试。

#### TypeScript Property Tests

```typescript
// tests/properties/message.property.test.ts
import { describe, it } from 'vitest';
import fc from 'fast-check';
import { validateMessage, appendMessage } from '@/utils/message';

describe('Message Properties', () => {
  it('Property: Non-empty messages should pass validation', () => {
    fc.assert(
      fc.property(
        fc.string({ minLength: 1 }).filter(s => s.trim().length > 0),
        (message) => {
          return validateMessage(message) === true;
        }
      ),
      { numRuns: 100 }
    );
  });
  
  it('Property: Whitespace-only messages should fail validation', () => {
    fc.assert(
      fc.property(
        fc.string().filter(s => s.length > 0 && s.trim().length === 0),
        (message) => {
          return validateMessage(message) === false;
        }
      ),
      { numRuns: 100 }
    );
  });
  
  it('Property: Appending message increases list length by 1', () => {
    fc.assert(
      fc.property(
        fc.array(fc.record({
          id: fc.uuid(),
          role: fc.constantFrom('user', 'assistant'),
          content: fc.string(),
          timestamp: fc.integer(),
        })),
        fc.record({
          id: fc.uuid(),
          role: fc.constantFrom('user', 'assistant'),
          content: fc.string(),
          timestamp: fc.integer(),
        }),
        (messages, newMessage) => {
          const originalLength = messages.length;
          const result = appendMessage(messages, newMessage);
          return result.length === originalLength + 1;
        }
      ),
      { numRuns: 100 }
    );
  });
});
```

#### Rust Property Tests

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn prop_conversation_limit_enforced(
            conversations in prop::collection::vec(
                any::<Conversation>(),
                0..200
            )
        ) {
            let limited = limit_conversations(conversations, 100);
            prop_assert!(limited.len() <= 100);
        }
        
        #[test]
        fn prop_context_length_limiting(
            messages in prop::collection::vec(
                any::<Message>(),
                0..50
            ),
            context_length in 0usize..20
        ) {
            let limited = limit_context(messages.clone(), context_length);
            prop_assert!(limited.len() <= context_length);
            prop_assert!(limited.len() <= messages.len());
        }
        
        #[test]
        fn prop_save_load_round_trip(
            conversation in any::<Conversation>()
        ) {
            let json = serde_json::to_string(&conversation).unwrap();
            let loaded: Conversation = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(conversation.id, loaded.id);
            prop_assert_eq!(conversation.messages.len(), loaded.messages.len());
        }
    }
}
```

### Integration Testing

**测试场景：**
1. 完整的消息发送流程（前端 → 后端 → API → 后端 → 前端）
2. 对话创建、切换、删除流程
3. 设置保存和加载
4. 错误恢复流程

**示例集成测试：**

```typescript
// tests/integration/chat-flow.test.ts
import { describe, it, expect, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

describe('Chat Flow Integration', () => {
  beforeEach(async () => {
    // 清理测试数据
    await invoke('clear_test_data');
  });
  
  it('should complete full message send flow', async () => {
    // 1. 创建对话
    const conversation = await invoke('create_conversation', {
      modelId: 'test-model',
    });
    
    expect(conversation.id).toBeTruthy();
    expect(conversation.messages).toHaveLength(0);
    
    // 2. 发送消息
    const response = await invoke('send_ai_message', {
      modelId: 'test-model',
      messages: [
        { role: 'user', content: 'Hello' },
      ],
      stream: false,
    });
    
    expect(response).toBeTruthy();
    
    // 3. 验证对话已更新
    const updated = await invoke('get_conversation', {
      id: conversation.id,
    });
    
    expect(updated.messages).toHaveLength(2);
    expect(updated.messages[0].role).toBe('user');
    expect(updated.messages[1].role).toBe('assistant');
  });
});
```

## Performance Considerations

### Frontend Optimization

#### 1. Virtual Scrolling

对于超过 50 条消息的对话，使用虚拟滚动减少 DOM 节点数量：

```typescript
import { useVirtualList } from '@vueuse/core';

const { list, containerProps, wrapperProps } = useVirtualList(
  messages,
  {
    itemHeight: 80, // 估算的消息高度
    overscan: 5,    // 预渲染的额外项数
  }
);
```

#### 2. Debounced Input

对用户输入进行防抖处理：

```typescript
import { useDebounceFn } from '@vueuse/core';

const debouncedInput = useDebounceFn((value: string) => {
  // 处理输入
}, 150);
```

#### 3. Lazy Loading

延迟加载 Markdown 渲染器和代码高亮库：

```typescript
const MarkdownRenderer = defineAsyncComponent(() =>
  import('@/components/MarkdownRenderer.vue')
);
```

#### 4. Memoization

缓存计算结果：

```typescript
const tokenCount = computed(() => {
  return messages.value.reduce((sum, msg) => {
    return sum + estimateTokens(msg.content);
  }, 0);
});
```

### Backend Optimization

#### 1. Connection Pooling

复用 HTTP 连接：

```rust
lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();
}
```

#### 2. Async Processing

使用异步处理避免阻塞：

```rust
#[command]
async fn send_ai_message(
    model_id: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    window: Window,
) -> Result<String, String> {
    tokio::spawn(async move {
        // 异步处理
    });
}
```

#### 3. Streaming Response

使用流式响应减少首字延迟：

```rust
// 立即开始发送响应块
while let Some(chunk) = stream.next().await {
    window.emit("ai-message-chunk", chunk).ok();
}
```

## Security Considerations

### 1. API Key Protection

- API Key 存储在本地加密文件中
- 前端只显示部分 API Key（前 8 位 + 后 4 位）
- 不在日志中记录完整 API Key

### 2. Input Sanitization

- 对用户输入进行 HTML 转义
- 限制消息长度（最大 10,000 字符）
- 过滤恶意脚本

### 3. Rate Limiting

- 限制 API 调用频率
- 实现指数退避重试策略
- 显示剩余配额

### 4. Data Privacy

- 所有对话数据仅存储在本地
- 不上传到云端
- 用户可以随时删除数据

## Deployment Considerations

### 1. Dependencies

**Frontend:**
- `marked`: Markdown 解析
- `highlight.js`: 代码高亮
- `@vueuse/core`: Vue 工具函数

**Backend:**
- `reqwest`: HTTP 客户端
- `tokio`: 异步运行时
- `serde_json`: JSON 处理

### 2. Build Configuration

```toml
# Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "stream"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```json
// package.json
{
  "dependencies": {
    "marked": "^11.0.0",
    "highlight.js": "^11.9.0",
    "@vueuse/core": "^10.7.0"
  }
}
```

### 3. Storage Location

```
~/.local/share/com.lewen.my-utools/
├── ai-config.json          # AI 模型配置
└── ai-chat.json            # 对话历史
```

### 4. Migration Strategy

如果需要更新数据格式：

```rust
fn migrate_conversations(old_data: Vec<OldConversation>) -> Vec<Conversation> {
    old_data.into_iter().map(|old| {
        Conversation {
            id: old.id,
            title: old.title.unwrap_or_else(|| "Untitled".to_string()),
            messages: old.messages,
            model_id: old.model_id.unwrap_or_else(|| "default".to_string()),
            created_at: old.created_at,
            updated_at: old.updated_at.unwrap_or(old.created_at),
            settings: ConversationSettings::default(),
        }
    }).collect()
}
```

## Future Enhancements

### Phase 2 Features

1. **多模态支持**
   - 图片输入
   - 文件上传
   - 语音输入

2. **高级功能**
   - 对话分享
   - 导出为 Markdown
   - 对话模板

3. **AI 功能增强**
   - 函数调用 (Function Calling)
   - 工具使用 (Tool Use)
   - 多轮推理

4. **用户体验**
   - 主题自定义
   - 快捷键配置
   - 插件扩展

### Phase 3 Features

1. **协作功能**
   - 团队共享对话
   - 评论和标注
   - 版本历史

2. **分析功能**
   - 使用统计
   - 成本分析
   - 性能监控

3. **企业功能**
   - SSO 集成
   - 审计日志
   - 合规性支持
