//! AI 对话插件后端
//!
//! 职责：
//! - 加载 ai-config.json 中的模型配置，向 OpenAI 兼容端点发起流式请求
//! - 通过 `ai-message-chunk` / `ai-message-done` 事件向前端推送增量与完成状态
//! - 支持中断正在生成的请求（保留已生成内容由前端负责）
//! - 对话数据持久化到 ai-chat.json（tauri-plugin-store），最多保留 100 条对话

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{command, AppHandle, Emitter, State, Window};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_store::StoreExt;

/// 最多保留的对话数量
const MAX_CONVERSATIONS: usize = 100;

/// 单条对话消息（与前端 types/index.ts 的 AiChatMessage 对应，snake_case）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageRecord {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 对话级设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSettings {
    pub system_prompt: String,
    pub context_length: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i64>,
}

impl Default for ConversationSettings {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            context_length: 20,
            temperature: None,
            max_tokens: None,
        }
    }
}

/// 对话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessageRecord>,
    pub model_id: String,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub settings: ConversationSettings,
}

/// API 请求消息（仅 role + content）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChatMessage {
    pub role: String,
    pub content: String,
}

/// AI 模型配置（与 ai-config.json 的 models 数组对应；name/is_default 由前端使用，后端不读）
#[derive(Debug, Clone, Deserialize)]
pub struct AiModelConfig {
    pub id: String,
    #[serde(default)]
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub enabled: bool,
}

/// OpenAI 兼容 chat/completions 请求体
#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: &'a [ApiChatMessage],
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<i64>,
}

/// 流式响应 chunk（OpenAI 格式，仅提取需要的字段）
#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    #[serde(default)]
    choices: Vec<ChunkChoice>,
}

#[derive(Debug, Deserialize)]
struct ChunkChoice {
    #[serde(default)]
    delta: ChunkDelta,
}

#[derive(Debug, Default, Deserialize)]
struct ChunkDelta {
    #[serde(default)]
    content: Option<String>,
}

/// 发送给前端的增量事件
#[derive(Clone, Serialize)]
struct StreamChunkEvent<'a> {
    message_id: &'a str,
    content: &'a str,
}

/// 发送给前端的完成事件（status: ok | error | stopped）
#[derive(Clone, Serialize)]
struct StreamDoneEvent {
    message_id: String,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// 当前生成任务句柄，用于中断
#[derive(Default)]
pub struct AiGeneration {
    task: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

/// SSE 行处理结果
#[derive(Debug, PartialEq)]
enum SseOutcome {
    /// delta 文本
    Content(String),
    /// 收到 [DONE]
    Done,
    /// 服务端通过事件报错（如 Anthropic error 事件）
    Error(String),
    /// 空行/注释/无内容的 chunk
    Ignore,
}

/// API 协议类型：按模型 provider 字段选择
#[derive(Debug, Clone, Copy, PartialEq)]
enum ApiProtocol {
    /// OpenAI 兼容协议（openai / deepseek / openrouter / custom 等网关）
    OpenAiCompatible,
    /// Anthropic 原生 /v1/messages 协议
    Anthropic,
}

impl ApiProtocol {
    fn from_provider(provider: &str) -> Self {
        match provider {
            "anthropic" => ApiProtocol::Anthropic,
            _ => ApiProtocol::OpenAiCompatible,
        }
    }
}

/// 解析一行 SSE data 负载（OpenAI 兼容协议）
fn handle_sse_data(data: &str) -> SseOutcome {
    let data = data.trim();
    if data.is_empty() {
        return SseOutcome::Ignore;
    }
    if data == "[DONE]" {
        return SseOutcome::Done;
    }
    match serde_json::from_str::<ChatCompletionChunk>(data) {
        Ok(chunk) => match chunk.choices.into_iter().next() {
            Some(choice) => match choice.delta.content {
                Some(content) if !content.is_empty() => SseOutcome::Content(content),
                _ => SseOutcome::Ignore,
            },
            None => SseOutcome::Ignore,
        },
        // 忽略无法解析的行（keep-alive、事件注释等）
        Err(_) => SseOutcome::Ignore,
    }
}

/// Anthropic /v1/messages 请求体（system 为独立顶层字段、max_tokens 必填）
#[derive(Debug, Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<ApiChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
}

/// Anthropic SSE 事件（仅提取需要的字段）
#[derive(Debug, Deserialize)]
struct AnthropicEvent {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    delta: Option<AnthropicDelta>,
    #[serde(default)]
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

/// 解析一行 SSE data 负载（Anthropic 原生协议）
fn handle_anthropic_sse_data(data: &str) -> SseOutcome {
    let data = data.trim();
    if data.is_empty() {
        return SseOutcome::Ignore;
    }
    let event: AnthropicEvent = match serde_json::from_str(data) {
        Ok(e) => e,
        Err(_) => return SseOutcome::Ignore,
    };
    match event.kind.as_str() {
        // 正文增量：delta.type == "text_delta" 时携带 text
        "content_block_delta" => match event.delta {
            Some(delta) if delta.kind == "text_delta" => match delta.text {
                Some(text) if !text.is_empty() => SseOutcome::Content(text),
                _ => SseOutcome::Ignore,
            },
            // thinking_delta 等其他增量类型忽略
            _ => SseOutcome::Ignore,
        },
        "message_stop" => SseOutcome::Done,
        "error" => {
            let message = event
                .error
                .as_ref()
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("未知错误");
            SseOutcome::Error(format!("Anthropic 错误: {}", message))
        }
        // message_start / content_block_start / content_block_stop / ping 等
        _ => SseOutcome::Ignore,
    }
}

/// 限制对话数量：按 updated_at 保留最近的 MAX_CONVERSATIONS 条
fn limit_conversations(mut conversations: Vec<Conversation>) -> Vec<Conversation> {
    if conversations.len() <= MAX_CONVERSATIONS {
        return conversations;
    }
    conversations.sort_by_key(|c| c.updated_at);
    let overflow = conversations.len() - MAX_CONVERSATIONS;
    conversations.drain(0..overflow);
    conversations
}

/// 从 ai-config.json 加载模型配置
fn load_ai_models() -> Result<Vec<AiModelConfig>, String> {
    let path = super::get_ai_config_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取 AI 配置失败: {}", e))?;
    let config: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析 AI 配置失败: {}", e))?;
    let models = config
        .get("models")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    serde_json::from_value::<Vec<AiModelConfig>>(models).map_err(|e| format!("解析模型列表失败: {}", e))
}

/// 发送 AI 消息（流式）。
///
/// 立即返回，请求在后台任务中执行：
/// - `ai-message-chunk` { message_id, content } 逐段推送 delta
/// - `ai-message-done`  { message_id, status, error? } 标记结束
#[command]
pub async fn send_ai_message(
    model_id: String,
    message_id: String,
    messages: Vec<ApiChatMessage>,
    temperature: Option<f64>,
    max_tokens: Option<i64>,
    window: Window,
    generation: State<'_, AiGeneration>,
) -> Result<(), String> {
    // 同一时间只允许一个生成任务
    {
        let slot = generation.task.lock().unwrap();
        if let Some(handle) = slot.as_ref() {
            if !handle.is_finished() {
                return Err("已有正在生成的回复，请先停止".to_string());
            }
        }
    }

    // 只允许使用已启用的模型（Property 2/23）
    let models = load_ai_models()?;
    let model = models
        .into_iter()
        .find(|m| m.id == model_id && m.enabled)
        .ok_or_else(|| "模型不存在或未启用，请检查 AI 模型配置".to_string())?;

    let window = window.clone();
    let handle = tokio::spawn(async move {
        let done = run_streaming_request(
            &window,
            message_id,
            model,
            messages,
            temperature,
            max_tokens,
        )
        .await;
        let _ = window.emit("ai-message-done", &done);
    });

    *generation.task.lock().unwrap() = Some(handle);
    Ok(())
}

/// 中断当前生成。已推送的内容由前端保留。
#[command]
pub async fn stop_ai_generation(
    window: Window,
    generation: State<'_, AiGeneration>,
) -> Result<(), String> {
    let mut slot = generation.task.lock().unwrap();
    if let Some(handle) = slot.take() {
        if !handle.is_finished() {
            handle.abort();
            // 中断后任务本身不会再发事件，由这里补发 stopped
            let _ = window.emit(
                "ai-message-done",
                &StreamDoneEvent {
                    message_id: String::new(),
                    status: "stopped",
                    error: None,
                },
            );
        }
    }
    Ok(())
}

/// 构造错误完成事件
fn err_event(message_id: &str, error: String) -> StreamDoneEvent {
    StreamDoneEvent {
        message_id: message_id.to_string(),
        status: "error",
        error: Some(error),
    }
}

/// 执行流式请求并逐段推送事件
async fn run_streaming_request(
    window: &Window,
    message_id: String,
    model: AiModelConfig,
    messages: Vec<ApiChatMessage>,
    temperature: Option<f64>,
    max_tokens: Option<i64>,
) -> StreamDoneEvent {
    let message_id = message_id.as_str();

    let client = match reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => return err_event(message_id, format!("创建 HTTP 客户端失败: {}", e)),
    };

    // 按协议构建请求（URL / 认证头 / 请求体各不相同）
    let protocol = ApiProtocol::from_provider(&model.provider);

    let request_builder = match protocol {
        ApiProtocol::OpenAiCompatible => {
            let url = format!("{}/chat/completions", model.base_url.trim_end_matches('/'));
            let body = ChatCompletionRequest {
                model: &model.model,
                messages: &messages,
                stream: true,
                temperature,
                max_tokens,
            };
            client
                .post(&url)
                .header("Authorization", format!("Bearer {}", model.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
        }
        ApiProtocol::Anthropic => {
            let url = format!("{}/messages", model.base_url.trim_end_matches('/'));
            // Anthropic 协议中 system 是独立顶层字段，不放在 messages 里
            let system: Vec<&str> = messages
                .iter()
                .filter(|m| m.role == "system")
                .map(|m| m.content.as_str())
                .collect();
            let chat_messages: Vec<ApiChatMessage> = messages
                .iter()
                .filter(|m| m.role != "system")
                .cloned()
                .collect();
            let body = AnthropicRequest {
                model: &model.model,
                // max_tokens 在 Anthropic 协议中必填
                max_tokens: max_tokens.unwrap_or(4096),
                system: if system.is_empty() {
                    None
                } else {
                    Some(system.join("\n\n"))
                },
                messages: chat_messages,
                stream: true,
                temperature,
            };
            client
                .post(&url)
                .header("x-api-key", &model.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
        }
    };

    let response = match request_builder.send().await {
        Ok(r) => r,
        Err(e) => return err_event(message_id, format!("网络请求失败，请检查网络或模型配置: {}", e)),
    };

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        let detail: String = text.chars().take(300).collect();
        return err_event(message_id, format!("API 错误 {}: {}", status, detail));
    }

    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    // SSE 数据可能跨 chunk 边界，必须缓冲后按行切分
    let mut buffer = String::new();

    // 按协议选择 data 负载解析器
    let parse_sse = |data: &str| match protocol {
        ApiProtocol::OpenAiCompatible => handle_sse_data(data),
        ApiProtocol::Anthropic => handle_anthropic_sse_data(data),
    };

    loop {
        match stream.next().await {
            Some(Ok(bytes)) => {
                buffer.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(pos) = buffer.find('\n') {
                    let line: String = buffer.drain(..=pos).collect();
                    let line = line.trim_end_matches(['\r', '\n']);

                    let Some(data) = line.strip_prefix("data:") else {
                        continue; // 忽略 event:/id:/注释/空行
                    };

                    match parse_sse(data) {
                        SseOutcome::Content(content) => {
                            let _ = window.emit(
                                "ai-message-chunk",
                                &StreamChunkEvent {
                                    message_id: &message_id,
                                    content: &content,
                                },
                            );
                        }
                        SseOutcome::Done => {
                            return StreamDoneEvent {
                                message_id: message_id.to_string(),
                                status: "ok",
                                error: None,
                            }
                        }
                        SseOutcome::Error(error) => {
                            return err_event(message_id, error);
                        }
                        SseOutcome::Ignore => {}
                    }
                }
            }
            Some(Err(e)) => {
                return err_event(message_id, format!("流式读取失败: {}", e));
            }
            // 流正常结束但未收到 [DONE]，同样视为成功
            None => {
                return StreamDoneEvent {
                    message_id: message_id.to_string(),
                    status: "ok",
                    error: None,
                }
            }
        }
    }
}

/// 加载所有对话
#[command]
pub async fn load_conversations(app: AppHandle) -> Result<Vec<Conversation>, String> {
    let store = app.store("ai-chat.json").map_err(|e| e.to_string())?;
    if let Some(list) = store.get("conversations") {
        // 数据损坏时返回空列表，由前端创建新对话并提示（Requirement 6.4）
        if let Ok(items) = serde_json::from_value::<Vec<Conversation>>(list) {
            return Ok(items);
        }
    }
    Ok(Vec::new())
}

/// 保存（新增或更新）对话，超限时淘汰最旧的
#[command]
pub async fn save_conversation(conversation: Conversation, app: AppHandle) -> Result<(), String> {
    let store = app.store("ai-chat.json").map_err(|e| e.to_string())?;

    let mut list: Vec<Conversation> = if let Some(v) = store.get("conversations") {
        serde_json::from_value(v).unwrap_or_default()
    } else {
        Vec::new()
    };

    if let Some(index) = list.iter().position(|c| c.id == conversation.id) {
        list[index] = conversation;
    } else {
        list.push(conversation);
    }

    let list = limit_conversations(list);
    store
        .set("conversations", serde_json::to_value(&list).map_err(|e| e.to_string())?);
    store.save().map_err(|e| format!("持久化失败: {}", e))?;
    Ok(())
}

/// 删除对话
#[command]
pub async fn delete_conversation(id: String, app: AppHandle) -> Result<(), String> {
    let store = app.store("ai-chat.json").map_err(|e| e.to_string())?;
    if let Some(v) = store.get("conversations") {
        if let Ok(mut list) = serde_json::from_value::<Vec<Conversation>>(v) {
            list.retain(|c| c.id != id);
            store
                .set("conversations", serde_json::to_value(&list).map_err(|e| e.to_string())?);
            store.save().map_err(|e| format!("持久化失败: {}", e))?;
        }
    }
    Ok(())
}

/// 在系统浏览器中打开链接（用于 Markdown 中的可点击链接）
#[command]
pub async fn open_external_url(url: String, app: AppHandle) -> Result<(), String> {
    // 仅允许 http/https，防止任意协议
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("仅支持 http/https 链接".to_string());
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| format!("打开链接失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_body_serialization() {
        let messages = vec![ApiChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        let req = ChatCompletionRequest {
            model: "gpt-4",
            messages: &messages,
            stream: true,
            temperature: Some(0.7),
            max_tokens: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""model":"gpt-4""#));
        assert!(json.contains(r#""stream":true"#));
        assert!(json.contains(r#""temperature":0.7"#));
        // None 字段不序列化
        assert!(!json.contains("max_tokens"));
    }

    #[test]
    fn test_parse_stream_chunk_content() {
        let json = r#"{"id":"123","object":"chat.completion.chunk","created":1234567890,"model":"gpt-4","choices":[{"index":0,"delta":{"role":"assistant","content":"Hello"},"finish_reason":null}]}"#;
        assert_eq!(
            handle_sse_data(json),
            SseOutcome::Content("Hello".to_string())
        );
    }

    #[test]
    fn test_parse_stream_chunk_empty_delta() {
        let json = r#"{"id":"123","choices":[{"index":0,"delta":{"role":"assistant"},"finish_reason":"stop"}]}"#;
        assert_eq!(handle_sse_data(json), SseOutcome::Ignore);
    }

    #[test]
    fn test_parse_stream_done() {
        assert_eq!(handle_sse_data("[DONE]"), SseOutcome::Done);
        assert_eq!(handle_sse_data("  [DONE]  "), SseOutcome::Done);
    }

    #[test]
    fn test_parse_invalid_line_ignored() {
        assert_eq!(handle_sse_data(": keep-alive"), SseOutcome::Ignore);
        assert_eq!(handle_sse_data("not json"), SseOutcome::Ignore);
        assert_eq!(handle_sse_data(""), SseOutcome::Ignore);
    }

    fn make_conversation(id: &str, updated_at: i64) -> Conversation {
        Conversation {
            id: id.to_string(),
            title: format!("Conversation {}", id),
            messages: vec![],
            model_id: "model-1".to_string(),
            created_at: updated_at,
            updated_at,
            settings: ConversationSettings::default(),
        }
    }

    #[test]
    fn test_conversation_limit_keeps_most_recent() {
        let mut conversations: Vec<Conversation> = (0..150)
            .map(|i| make_conversation(&format!("conv-{}", i), i))
            .collect();

        conversations = limit_conversations(conversations);

        assert_eq!(conversations.len(), MAX_CONVERSATIONS);
        assert_eq!(conversations[0].id, "conv-50");
        assert_eq!(conversations[99].id, "conv-149");
    }

    #[test]
    fn test_conversation_limit_noop_when_under() {
        let conversations = vec![
            make_conversation("a", 1),
            make_conversation("b", 2),
        ];
        let result = limit_conversations(conversations);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_conversation_serde_round_trip() {
        let conversation = Conversation {
            id: "c1".to_string(),
            title: "测试".to_string(),
            messages: vec![ChatMessageRecord {
                id: "m1".to_string(),
                role: "user".to_string(),
                content: "你好".to_string(),
                timestamp: 1700000000,
                tokens: Some(2),
                error: None,
            }],
            model_id: "model-1".to_string(),
            created_at: 1700000000,
            updated_at: 1700000001,
            settings: ConversationSettings {
                system_prompt: "你是助手".to_string(),
                context_length: 10,
                temperature: None,
                max_tokens: None,
            },
        };
        let json = serde_json::to_string(&conversation).unwrap();
        let loaded: Conversation = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.id, conversation.id);
        assert_eq!(loaded.messages.len(), 1);
        assert_eq!(loaded.messages[0].content, "你好");
        assert_eq!(loaded.settings.context_length, 10);
    }

    #[test]
    fn test_conversation_default_settings_on_missing() {
        // 旧数据没有 settings 字段时应使用默认值
        let json = r#"{"id":"c1","title":"t","messages":[],"model_id":"m","created_at":1,"updated_at":2}"#;
        let loaded: Conversation = serde_json::from_str(json).unwrap();
        assert_eq!(loaded.settings.context_length, 20);
        assert!(loaded.settings.system_prompt.is_empty());
    }

    #[test]
    fn test_protocol_from_provider() {
        assert_eq!(
            ApiProtocol::from_provider("anthropic"),
            ApiProtocol::Anthropic
        );
        assert_eq!(
            ApiProtocol::from_provider("openai"),
            ApiProtocol::OpenAiCompatible
        );
        assert_eq!(
            ApiProtocol::from_provider("deepseek"),
            ApiProtocol::OpenAiCompatible
        );
        assert_eq!(
            ApiProtocol::from_provider("custom"),
            ApiProtocol::OpenAiCompatible
        );
        assert_eq!(
            ApiProtocol::from_provider(""),
            ApiProtocol::OpenAiCompatible
        );
    }

    #[test]
    fn test_anthropic_request_serialization() {
        let request = AnthropicRequest {
            model: "claude-sonnet-4",
            max_tokens: 4096,
            system: Some("你是翻译引擎".to_string()),
            messages: vec![
                ApiChatMessage {
                    role: "user".to_string(),
                    content: "hello".to_string(),
                },
                ApiChatMessage {
                    role: "assistant".to_string(),
                    content: "hi".to_string(),
                },
            ],
            stream: true,
            temperature: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        // system 为顶层字段
        assert!(json.contains(r#""system":"你是翻译引擎""#));
        // max_tokens 必填
        assert!(json.contains(r#""max_tokens":4096"#));
        assert!(json.contains(r#""stream":true"#));
        // None 的 temperature 不序列化
        assert!(!json.contains("temperature"));
        // messages 只含 user/assistant
        assert!(!json.contains(r#""role":"system""#));
    }

    #[test]
    fn test_anthropic_sse_content_delta() {
        let json = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"你好"}}"#;
        assert_eq!(
            handle_anthropic_sse_data(json),
            SseOutcome::Content("你好".to_string())
        );
    }

    #[test]
    fn test_anthropic_sse_non_text_delta_ignored() {
        // 思考增量等非正文类型不作为内容输出
        let json = r#"{"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"..."}}"#;
        assert_eq!(handle_anthropic_sse_data(json), SseOutcome::Ignore);
    }

    #[test]
    fn test_anthropic_sse_lifecycle_events() {
        assert_eq!(
            handle_anthropic_sse_data(r#"{"type":"message_stop"}"#),
            SseOutcome::Done
        );
        assert_eq!(handle_anthropic_sse_data(r#"{"type":"ping"}"#), SseOutcome::Ignore);
        assert_eq!(
            handle_anthropic_sse_data(r#"{"type":"message_start","message":{}}"#),
            SseOutcome::Ignore
        );
        assert_eq!(
            handle_anthropic_sse_data(r#"{"type":"content_block_start","index":0}"#),
            SseOutcome::Ignore
        );
        assert_eq!(handle_anthropic_sse_data(""), SseOutcome::Ignore);
        assert_eq!(handle_anthropic_sse_data("not json"), SseOutcome::Ignore);
    }

    #[test]
    fn test_anthropic_sse_error_event() {
        let json = r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#;
        match handle_anthropic_sse_data(json) {
            SseOutcome::Error(msg) => assert!(msg.contains("Overloaded")),
            other => panic!("expected Error, got {:?}", other),
        }
    }
}
