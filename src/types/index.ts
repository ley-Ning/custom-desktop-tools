export interface App {
    id: string;
    name: string;
    path: string;
    icon?: string;
}

export interface Plugin {
    id: string;
    name: string;
    icon: string;
    gradient: string;
    keywords: string[];
    description: string;
    enabled: boolean;
    builtin: boolean; // 是否为内置插件
    version?: string;
    author?: string;
}

export interface PluginConfig {
    id: string;
    name: string;
    version: string;
    main: string;
    icon?: string;
    keywords?: string[];
    description?: string;
}

// ===== AI 对话插件（与 src-tauri/src/ai_chat.rs 的结构一一对应，snake_case） =====

export type MessageRole = 'user' | 'assistant' | 'system';

/** AI 模型配置（ai-config.json 中的条目） */
export interface AiModel {
    id: string;
    name: string;
    provider: string;
    baseUrl: string;
    apiKey: string;
    model: string;
    enabled: boolean;
    isDefault: boolean;
}

/** 对话中的单条消息 */
export interface AiChatMessage {
    id: string;
    role: MessageRole;
    content: string;
    timestamp: number;
    tokens?: number;
    error?: string;
}

/** 对话级设置 */
export interface ConversationSettings {
    system_prompt: string;
    context_length: number;
    temperature?: number;
    max_tokens?: number;
}

/** 一次对话会话 */
export interface Conversation {
    id: string;
    title: string;
    messages: AiChatMessage[];
    model_id: string;
    created_at: number;
    updated_at: number;
    settings: ConversationSettings;
}

/** API 请求消息（仅 role + content） */
export interface ApiChatMessage {
    role: MessageRole;
    content: string;
}

/** 后端 ai-message-chunk 事件负载 */
export interface StreamChunkEvent {
    message_id: string;
    content: string;
}

/** 后端 ai-message-done 事件负载（status: ok | error | stopped） */
export interface StreamDoneEvent {
    message_id: string;
    status: 'ok' | 'error' | 'stopped';
    error?: string;
}

// ===== 计算稿纸插件（与 src-tauri/src/calc.rs 对应，snake_case） =====

/** 稿纸单行求值结果 */
export interface CalcLineResult {
    input: string;
    ok: boolean;
    value?: string;
    error?: string;
    is_assignment: boolean;
    var_name?: string;
}
