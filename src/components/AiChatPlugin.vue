<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import dayjs from "dayjs";
import { marked } from "marked";
import DOMPurify from "dompurify";
import hljs from "highlight.js/lib/common";
import type {
  AiModel,
  AiChatMessage,
  Conversation,
  ConversationSettings,
  ApiChatMessage,
  StreamChunkEvent,
  StreamDoneEvent,
} from "../types";

const props = defineProps<{
  initialInput?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

// ===== 状态 =====

const conversations = ref<Conversation[]>([]);
const currentId = ref("");
const input = ref("");
const isGenerating = ref(false);
const streamingMessageId = ref<string | null>(null);

const models = ref<AiModel[]>([]);
const selectedModelId = ref("");

const showSettingsPanel = ref(false);
const showDeleteConfirm = ref(false);
const deleteTargetId = ref("");
const showClearConfirm = ref(false);

const copiedMessageId = ref("");
let copiedTimer: number | null = null;

const inputEl = ref<HTMLTextAreaElement | null>(null);
const messagesEl = ref<HTMLElement | null>(null);
let stickToBottom = true;

// ===== Markdown 渲染（marked + highlight.js + DOMPurify） =====

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

marked.use({
  gfm: true,
  breaks: true,
  renderer: {
    code(token: { text: string; lang?: string }): string {
      const language = (token.lang || "").split(/\s+/)[0];
      let html: string;
      if (language && hljs.getLanguage(language)) {
        try {
          html = hljs.highlight(token.text, { language, ignoreIllegals: true }).value;
        } catch {
          html = escapeHtml(token.text);
        }
      } else {
        html = escapeHtml(token.text);
      }
      return `<pre class="code-block"><code class="hljs">${html}</code></pre>`;
    },
  },
});

// 渲染缓存：流式更新时只有内容变化的消息需要重新渲染
const renderCache = new Map<string, string>();

function renderMarkdown(message: AiChatMessage): string {
  if (!message.content) return "";
  const key = `${message.id}:${message.content.length}:${message.error ? 1 : 0}`;
  const cached = renderCache.get(key);
  if (cached !== undefined) return cached;

  const raw = marked.parse(message.content, { async: false }) as string;
  const clean = DOMPurify.sanitize(raw, { ADD_ATTR: ["target"] });
  // 限制缓存体积，避免超长对话内存膨胀
  if (renderCache.size > 300) renderCache.clear();
  renderCache.set(key, clean);
  return clean;
}

// token 用量估算（CJK 约 1.5 字/token，其余约 4 字符/token）
function estimateTokens(text: string): number {
  const cjk = (text.match(/[\u4e00-\u9fa5\u3040-\u30ff\uff00-\uffef]/g) || []).length;
  return Math.ceil(cjk * 1.5 + (text.length - cjk) / 4);
}

// ===== 计算属性 =====

const currentConversation = computed(
  () => conversations.value.find((c) => c.id === currentId.value) || null
);

const selectedModel = computed(
  () => models.value.find((m) => m.id === selectedModelId.value) || null
);

const hasModels = computed(() => models.value.length > 0);

const totalTokens = computed(() => {
  if (!currentConversation.value) return 0;
  return currentConversation.value.messages.reduce(
    (sum, msg) => sum + (msg.content ? estimateTokens(msg.content) : 0),
    0
  );
});

// ===== 对话管理 =====

function defaultSettings(): ConversationSettings {
  return { system_prompt: "", context_length: 20 };
}

function newId(): string {
  return crypto.randomUUID();
}

function createConversation(): Conversation {
  const now = Math.floor(Date.now() / 1000);
  return {
    id: newId(),
    title: "新对话",
    messages: [],
    model_id: selectedModelId.value,
    created_at: now,
    updated_at: now,
    settings: defaultSettings(),
  };
}

async function persistConversation(conversation: Conversation) {
  try {
    await invoke("save_conversation", { conversation });
  } catch (e) {
    // 保存失败不影响当前对话（Requirement 6.3）
    console.error("保存对话失败:", e);
  }
}

function selectConversation(id: string) {
  if (id === currentId.value) return;
  closePanels();
  currentId.value = id;
  const conversation = conversations.value.find((c) => c.id === id);
  // 恢复该对话绑定的模型（若仍可用）
  if (conversation?.model_id && models.value.some((m) => m.id === conversation.model_id)) {
    selectedModelId.value = conversation.model_id;
  }
  scrollToBottom(true);
}

function handleNewConversation() {
  closePanels();
  const conversation = createConversation();
  conversations.value.unshift(conversation);
  currentId.value = conversation.id;
  persistConversation(conversation);
  scrollToBottom(true);
  focusInput();
}

function requestDeleteConversation(id: string) {
  deleteTargetId.value = id;
  showDeleteConfirm.value = true;
}

async function confirmDeleteConversation() {
  const id = deleteTargetId.value;
  showDeleteConfirm.value = false;
  deleteTargetId.value = "";

  const index = conversations.value.findIndex((c) => c.id === id);
  if (index === -1) return;
  conversations.value.splice(index, 1);

  try {
    await invoke("delete_conversation", { id });
  } catch (e) {
    console.error("删除对话失败:", e);
  }

  // 列表为空时自动新建（Requirement 5.5）
  if (conversations.value.length === 0) {
    handleNewConversation();
  } else if (currentId.value === id) {
    currentId.value = conversations.value[Math.max(0, index - 1)].id;
    scrollToBottom(true);
  }
}

function requestClearConversation() {
  if (!currentConversation.value?.messages.length) return;
  showClearConfirm.value = true;
}

async function confirmClearConversation() {
  showClearConfirm.value = false;
  const conversation = currentConversation.value;
  if (!conversation) return;
  // 清空消息但保留模型选择（Requirement 9.4）
  conversation.messages = [];
  conversation.title = "新对话";
  conversation.updated_at = Math.floor(Date.now() / 1000);
  renderCache.clear();
  await persistConversation(conversation);
  scrollToBottom(true);
}

function closePanels() {
  showSettingsPanel.value = false;
  showDeleteConfirm.value = false;
  showClearConfirm.value = false;
}

// ===== 模型管理 =====

async function loadModels() {
  try {
    const configStr = await invoke<string>("load_ai_config");
    if (configStr) {
      const config = JSON.parse(configStr) as { models?: AiModel[] };
      // 仅显示已启用的模型（Requirement 2.2）
      models.value = (config.models || []).filter((m) => m.enabled);
    }
  } catch (e) {
    console.error("加载 AI 模型配置失败:", e);
    models.value = [];
  }

  if (models.value.length > 0) {
    const conversation = currentConversation.value;
    const preferred =
      (conversation?.model_id && models.value.find((m) => m.id === conversation.model_id)) ||
      models.value.find((m) => m.isDefault) ||
      models.value[0];
    selectedModelId.value = preferred.id;
  }
}

function onModelChange() {
  const conversation = currentConversation.value;
  if (conversation) {
    conversation.model_id = selectedModelId.value;
    persistConversation(conversation);
  }
}

// ===== 消息发送 =====

function buildContextMessages(conversation: Conversation): ApiChatMessage[] {
  const { system_prompt, context_length } = conversation.settings;
  const messages: ApiChatMessage[] = [];

  if (system_prompt.trim()) {
    messages.push({ role: "system", content: system_prompt.trim() });
  }

  // 只取最近 N 条有效历史（Requirement 11）
  const valid = conversation.messages.filter((m) => !m.error && m.content.trim());
  const recent = context_length > 0 ? valid.slice(-context_length) : [];
  for (const msg of recent) {
    messages.push({ role: msg.role, content: msg.content });
  }
  return messages;
}

async function sendMessage() {
  const content = input.value.trim();
  if (!content || isGenerating.value) return; // 空消息阻止发送（Requirement 3.5）
  if (!selectedModel.value) return;

  const conversation = currentConversation.value;
  if (!conversation) return;

  closePanels();

  // 追加用户消息（Requirement 4.1）
  const userMessage: AiChatMessage = {
    id: newId(),
    role: "user",
    content,
    timestamp: Math.floor(Date.now() / 1000),
  };
  conversation.messages.push(userMessage);

  // 首条用户消息自动生成标题
  if (conversation.title === "新对话") {
    conversation.title = content.length > 20 ? `${content.slice(0, 20)}…` : content;
  }

  input.value = "";
  nextTick(() => autoResize());
  await persistConversation(conversation);
  scrollToBottom(true);

  await requestAssistantReply(conversation);
}

async function requestAssistantReply(conversation: Conversation) {
  const assistantMessage: AiChatMessage = {
    id: newId(),
    role: "assistant",
    content: "",
    timestamp: Math.floor(Date.now() / 1000),
  };
  conversation.messages.push(assistantMessage);
  conversation.updated_at = assistantMessage.timestamp;

  streamingMessageId.value = assistantMessage.id;
  isGenerating.value = true;
  stickToBottom = true;
  scrollToBottom(true);

  const messages = buildContextMessages(conversation);
  try {
    // 命令立即返回，回复通过 ai-message-chunk / ai-message-done 事件推送
    await invoke("send_ai_message", {
      modelId: selectedModelId.value,
      messageId: assistantMessage.id,
      messages,
    });
  } catch (e) {
    // 启动失败（无可用模型 / 已有生成任务等）
    finalizeStream(assistantMessage.id, "error", String(e));
  }
}

function findMessage(id: string): { conversation: Conversation; message: AiChatMessage } | null {
  for (const conversation of conversations.value) {
    const message = conversation.messages.find((m) => m.id === id);
    if (message) return { conversation, message };
  }
  return null;
}

function finalizeStream(messageId: string, status: "ok" | "error" | "stopped", error?: string) {
  const found = findMessage(messageId);
  if (found) {
    const { conversation, message } = found;
    if (status === "error") {
      message.error = error || "请求失败";
      message.tokens = message.content ? estimateTokens(message.content) : undefined;
    }
    conversation.updated_at = Math.floor(Date.now() / 1000);
    persistConversation(conversation);
  }
  streamingMessageId.value = null;
  isGenerating.value = false;
  focusInput();
}

async function stopGeneration() {
  if (!isGenerating.value) return;
  try {
    await invoke("stop_ai_generation");
    // stopped 事件由 stop 命令补发，前端在 done 监听中统一收尾
  } catch (e) {
    console.error("停止生成失败:", e);
    if (streamingMessageId.value) {
      finalizeStream(streamingMessageId.value, "stopped");
    }
  }
}

function retryLastMessage() {
  const conversation = currentConversation.value;
  if (!conversation || isGenerating.value) return;

  // 移除末尾连续的失败回复后重新请求
  while (conversation.messages.length > 0) {
    const last = conversation.messages[conversation.messages.length - 1];
    if (last.role === "assistant" && last.error) {
      conversation.messages.pop();
    } else {
      break;
    }
  }
  if (conversation.messages.length === 0) return;
  requestAssistantReply(conversation);
}

// ===== 事件监听（后端推送） =====

let unlistenChunk: (() => void) | null = null;
let unlistenDone: (() => void) | null = null;

async function setupListeners() {
  unlistenChunk = await listen<StreamChunkEvent>("ai-message-chunk", (event) => {
    const messageId = streamingMessageId.value;
    if (!messageId || event.payload.message_id !== messageId) return;
    const found = findMessage(messageId);
    if (!found) return;
    found.message.content += event.payload.content;
    scrollToBottom();
  });

  unlistenDone = await listen<StreamDoneEvent>("ai-message-done", (event) => {
    const messageId = streamingMessageId.value;
    if (!messageId) return;
    const { message_id, status, error } = event.payload;
    // stop 命令补发的 stopped 事件 message_id 为空，匹配当前流式消息
    if (message_id && message_id !== messageId) return;
    finalizeStream(messageId, status, error);
  });
}

// ===== UI 交互 =====

function formatTime(timestamp: number): string {
  return dayjs(timestamp * 1000).format("HH:mm");
}

async function copyMessage(message: AiChatMessage) {
  try {
    await invoke("write_clipboard_text", { text: message.content });
    copiedMessageId.value = message.id;
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedTimer = window.setTimeout(() => {
      copiedMessageId.value = "";
    }, 1500);
  } catch (e) {
    console.error("复制失败:", e);
  }
}

function onInputKeydown(e: KeyboardEvent) {
  // Enter 发送 / Shift+Enter 换行（Requirement 3.1 / 3.2）
  if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
    e.preventDefault();
    sendMessage();
  }
}

function autoResize() {
  const el = inputEl.value;
  if (!el) return;
  el.style.height = "auto";
  el.style.height = `${Math.min(el.scrollHeight, 120)}px`;
}

function onScroll() {
  const el = messagesEl.value;
  if (!el) return;
  // 用户上滑查看历史时不自动跳转（Requirement 4.3）
  stickToBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 60;
}

function scrollToBottom(force = false) {
  if (!force && !stickToBottom) return;
  nextTick(() => {
    const el = messagesEl.value;
    if (el) el.scrollTop = el.scrollHeight;
  });
}

// Markdown 内链接点击 → 系统浏览器打开
function onMessageClick(e: MouseEvent) {
  const anchor = (e.target as HTMLElement).closest("a");
  if (anchor) {
    e.preventDefault();
    const href = anchor.getAttribute("href") || "";
    if (href.startsWith("http://") || href.startsWith("https://")) {
      invoke("open_external_url", { url: href }).catch((err) =>
        console.error("打开链接失败:", err)
      );
    }
  }
}

function focusInput() {
  nextTick(() => inputEl.value?.focus());
}

// ===== 生命周期 =====

onMounted(async () => {
  // 加载对话历史（Requirement 6.2；损坏时后端返回空列表）
  try {
    const list = await invoke<Conversation[]>("load_conversations");
    conversations.value = Array.isArray(list) ? list : [];
  } catch (e) {
    console.error("加载对话历史失败:", e);
    conversations.value = [];
  }

  // 为空时自动创建新对话（Requirement 5.5）
  if (conversations.value.length === 0) {
    conversations.value = [createConversation()];
    persistConversation(conversations.value[0]);
  }
  currentId.value = conversations.value[0].id;

  await loadModels();
  await setupListeners();

  // 搜索框带入的初始输入（如智能推荐进入）
  if (props.initialInput?.trim()) {
    input.value = props.initialInput;
  }

  // 生成中按 Esc 先停止生成而不是关闭插件（捕获阶段拦截主窗口的 Esc 处理）
  window.addEventListener("keydown", onCaptureKeydown, true);

  focusInput();
  scrollToBottom(true);
});

function onCaptureKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && isGenerating.value) {
    e.preventDefault();
    e.stopImmediatePropagation();
    stopGeneration();
  }
}

onUnmounted(() => {
  window.removeEventListener("keydown", onCaptureKeydown, true);
  if (unlistenChunk) unlistenChunk();
  if (unlistenDone) unlistenDone();
  if (copiedTimer) clearTimeout(copiedTimer);
  // 组件销毁时中断未完成的生成
  if (isGenerating.value) {
    invoke("stop_ai_generation").catch(() => {});
  }
});
</script>

<template>
  <div class="ai-chat-plugin">
    <!-- 顶栏 -->
    <div class="chat-header">
      <button class="back-btn" title="返回" @click="emit('close')">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M19 12H5" />
          <path d="M12 19l-7-7 7-7" />
        </svg>
      </button>

      <div class="header-title">AI 对话</div>

      <select
        v-if="hasModels"
        v-model="selectedModelId"
        class="model-select"
        title="选择模型"
        @change="onModelChange"
      >
        <option v-for="model in models" :key="model.id" :value="model.id">
          {{ model.name }}
        </option>
      </select>
      <div v-else class="no-model-hint">未配置模型</div>

      <div class="header-actions">
        <button class="header-btn" title="新建对话" @click="handleNewConversation">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 5v14M5 12h14" />
          </svg>
        </button>
        <button
          class="header-btn"
          :class="{ active: showSettingsPanel }"
          title="对话设置"
          @click="showSettingsPanel = !showSettingsPanel"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h.01a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51h.01a1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v.01a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
        <button class="header-btn" title="清空对话" @click="requestClearConversation">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 6h18" />
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
            <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          </svg>
        </button>
      </div>
    </div>

    <div class="chat-body">
      <!-- 会话列表 -->
      <div class="conversation-sidebar">
        <div class="sidebar-header">对话</div>
        <div class="conversation-list">
          <div
            v-for="conversation in conversations"
            :key="conversation.id"
            class="conversation-item"
            :class="{ active: conversation.id === currentId }"
            @click="selectConversation(conversation.id)"
          >
            <div class="conversation-title">{{ conversation.title }}</div>
            <button
              class="conversation-delete"
              title="删除对话"
              @click.stop="requestDeleteConversation(conversation.id)"
            >
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- 主聊天区 -->
      <div class="chat-main">
        <!-- 未配置模型引导（Requirement 2.4） -->
        <div v-if="!hasModels" class="empty-state">
          <div class="empty-icon">🤖</div>
          <div class="empty-title">尚未配置 AI 模型</div>
          <div class="empty-desc">
            请在主界面进入 <b>设置 → AI 模型配置</b>，添加并启用模型后即可开始对话。
          </div>
        </div>

        <template v-else>
          <!-- 消息列表 -->
          <div ref="messagesEl" class="messages" @scroll="onScroll" @click="onMessageClick">
            <!-- 欢迎信息（Requirement 4.4） -->
            <div v-if="!currentConversation?.messages.length" class="welcome">
              <div class="welcome-icon">🤖</div>
              <div class="welcome-title">开始新对话</div>
              <div class="welcome-tips">
                <div class="tip">Enter 发送消息，Shift + Enter 换行</div>
                <div class="tip">支持 Markdown 与代码高亮渲染</div>
                <div class="tip">在 ⚙ 设置中可自定义系统提示词与上下文长度</div>
              </div>
            </div>

            <template v-for="message in currentConversation?.messages" :key="message.id">
              <div v-if="message.role === 'user'" class="message-item user">
                <div class="message-content user-bubble">{{ message.content }}</div>
                <div class="message-meta">
                  <span>{{ formatTime(message.timestamp) }}</span>
                  <button
                    class="copy-btn"
                    :class="{ copied: copiedMessageId === message.id }"
                    title="复制"
                    @click="copyMessage(message)"
                  >
                    {{ copiedMessageId === message.id ? "已复制" : "复制" }}
                  </button>
                </div>
              </div>

              <div v-else class="message-item assistant">
                <div class="assistant-avatar">🤖</div>
                <div class="message-body">
                  <!-- 生成中且尚无内容 -->
                  <div
                    v-if="!message.content && !message.error && message.id === streamingMessageId"
                    class="typing-indicator"
                  >
                    <span></span><span></span><span></span>
                  </div>

                  <div
                    v-else-if="message.content"
                    class="message-content markdown-body"
                    :class="{ streaming: message.id === streamingMessageId }"
                    v-html="renderMarkdown(message)"
                  ></div>

                  <!-- 错误信息 + 重试（Requirement 3.6） -->
                  <div v-if="message.error" class="message-error">
                    <span class="error-text">{{ message.error }}</span>
                    <button class="retry-btn" :disabled="isGenerating" @click="retryLastMessage">
                      重试
                    </button>
                  </div>

                  <div v-if="message.content && message.id !== streamingMessageId" class="message-meta">
                    <span>{{ formatTime(message.timestamp) }}</span>
                    <button
                      class="copy-btn"
                      :class="{ copied: copiedMessageId === message.id }"
                      title="复制"
                      @click="copyMessage(message)"
                    >
                      {{ copiedMessageId === message.id ? "已复制" : "复制" }}
                    </button>
                  </div>
                </div>
              </div>
            </template>
          </div>

          <!-- 统计 + 输入区（Requirement 12） -->
          <div class="chat-input-area">
            <div class="stats-bar">
              <span>{{ currentConversation?.messages.length || 0 }} 条消息</span>
              <span>约 {{ totalTokens }} tokens</span>
            </div>
            <div class="input-row">
              <textarea
                ref="inputEl"
                v-model="input"
                class="chat-input"
                rows="1"
                placeholder="输入消息，Enter 发送，Shift + Enter 换行"
                :disabled="isGenerating || !hasModels"
                @keydown="onInputKeydown"
                @input="autoResize"
              ></textarea>
              <button
                v-if="isGenerating"
                class="stop-btn"
                title="停止生成"
                @click="stopGeneration"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                  <rect x="6" y="6" width="12" height="12" rx="2" />
                </svg>
                停止
              </button>
              <button
                v-else
                class="send-btn"
                :disabled="!input.trim() || !hasModels"
                title="发送"
                @click="sendMessage"
              >
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 2L11 13" />
                  <path d="M22 2l-7 20-4-9-9-4 20-7z" />
                </svg>
                发送
              </button>
            </div>
          </div>
        </template>

        <!-- 对话设置面板（Requirement 10 / 11） -->
        <div v-if="showSettingsPanel && currentConversation" class="settings-panel">
          <div class="settings-header">
            <span>对话设置</span>
            <button class="close-panel-btn" @click="showSettingsPanel = false">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="settings-field">
            <label class="field-label">系统提示词</label>
            <textarea
              v-model="currentConversation.settings.system_prompt"
              class="system-prompt-input"
              rows="4"
              placeholder="定义 AI 的行为和角色，留空使用默认行为"
            ></textarea>
          </div>

          <div class="settings-field">
            <label class="field-label">
              上下文长度：{{ currentConversation.settings.context_length === 0 ? "不携带历史" : `${currentConversation.settings.context_length} 条` }}
            </label>
            <input
              v-model.number="currentConversation.settings.context_length"
              type="range"
              min="0"
              max="50"
              step="1"
              class="context-slider"
            />
            <div class="slider-marks">
              <span>0</span>
              <span>50</span>
            </div>
          </div>
        </div>

        <!-- 删除确认（Requirement 5.3） -->
        <div v-if="showDeleteConfirm" class="confirm-overlay" @click.self="showDeleteConfirm = false">
          <div class="confirm-dialog">
            <div class="confirm-title">删除对话</div>
            <div class="confirm-desc">确定要删除这个对话吗？删除后无法恢复。</div>
            <div class="confirm-actions">
              <button class="cancel-btn" @click="showDeleteConfirm = false">取消</button>
              <button class="danger-btn" @click="confirmDeleteConversation">删除</button>
            </div>
          </div>
        </div>

        <!-- 清空确认（Requirement 9.1） -->
        <div v-if="showClearConfirm" class="confirm-overlay" @click.self="showClearConfirm = false">
          <div class="confirm-dialog">
            <div class="confirm-title">清空对话</div>
            <div class="confirm-desc">确定要清空当前对话的所有消息吗？</div>
            <div class="confirm-actions">
              <button class="cancel-btn" @click="showClearConfirm = false">取消</button>
              <button class="danger-btn" @click="confirmClearConversation">清空</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ai-chat-plugin {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: rgba(30, 30, 30, 0.92);
  backdrop-filter: blur(40px) saturate(180%);
  -webkit-backdrop-filter: blur(40px) saturate(180%);
  border-radius: 18px;
  overflow: hidden;
}

/* ===== 顶栏 ===== */
.chat-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  background: rgba(40, 40, 40, 0.9);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.back-btn,
.header-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.65);
  cursor: pointer;
  transition: all 150ms cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
}

.back-btn:hover,
.header-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.95);
}

.header-btn.active {
  background: #007aff;
  border-color: #007aff;
  color: #fff;
}

.header-title {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.95);
  flex-shrink: 0;
}

.model-select {
  max-width: 200px;
  padding: 5px 10px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.9);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  outline: none;
  cursor: pointer;
}

.model-select option {
  background: #2a2a2a;
  color: rgba(255, 255, 255, 0.9);
}

.no-model-hint {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.header-actions {
  display: flex;
  gap: 6px;
  margin-left: auto;
}

/* ===== 主体 ===== */
.chat-body {
  flex: 1;
  display: flex;
  min-height: 0;
}

/* 会话侧栏 */
.conversation-sidebar {
  width: 170px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: rgba(38, 38, 38, 0.7);
  border-right: 1px solid rgba(255, 255, 255, 0.06);
}

.sidebar-header {
  padding: 12px 14px 8px;
  font-size: 11px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.4);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.conversation-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 8px;
}

.conversation-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  margin-bottom: 2px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 120ms ease;
}

.conversation-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.conversation-item.active {
  background: rgba(0, 122, 255, 0.2);
}

.conversation-title {
  flex: 1;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.85);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.conversation-item.active .conversation-title {
  color: #fff;
}

.conversation-delete {
  display: none;
  width: 20px;
  height: 20px;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  flex-shrink: 0;
}

.conversation-item:hover .conversation-delete {
  display: flex;
}

.conversation-delete:hover {
  background: rgba(255, 82, 82, 0.25);
  color: #ff6b6b;
}

/* 聊天主区 */
.chat-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  position: relative;
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.messages::-webkit-scrollbar {
  width: 8px;
}

.messages::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
}

/* 超过 50 条消息时跳过屏外渲染（Requirement 14.3） */
.message-item {
  margin-bottom: 16px;
  content-visibility: auto;
  contain-intrinsic-size: auto 90px;
}

.message-item.user {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.user-bubble {
  max-width: 78%;
  padding: 9px 14px;
  font-size: 13px;
  line-height: 1.55;
  color: #fff;
  background: linear-gradient(135deg, #0a84ff, #0066d6);
  border-radius: 14px 14px 4px 14px;
  white-space: pre-wrap;
  word-break: break-word;
}

.message-item.assistant {
  display: flex;
  gap: 10px;
}

.assistant-avatar {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  border-radius: 9px;
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
  flex-shrink: 0;
}

.message-body {
  min-width: 0;
  max-width: calc(100% - 44px);
}

/* Markdown 渲染 */
.markdown-body {
  font-size: 13px;
  line-height: 1.65;
  color: rgba(255, 255, 255, 0.92);
  word-break: break-word;
}

.markdown-body :deep(p) {
  margin: 0 0 8px;
}

.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4) {
  margin: 12px 0 8px;
  font-weight: 600;
  color: #fff;
}

.markdown-body :deep(h1) { font-size: 17px; }
.markdown-body :deep(h2) { font-size: 15px; }
.markdown-body :deep(h3),
.markdown-body :deep(h4) { font-size: 14px; }

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 4px 0 8px;
  padding-left: 22px;
}

.markdown-body :deep(li) {
  margin-bottom: 3px;
}

.markdown-body :deep(a) {
  color: #5eb5ff;
  text-decoration: none;
  cursor: pointer;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(blockquote) {
  margin: 8px 0;
  padding: 4px 12px;
  border-left: 3px solid rgba(0, 122, 255, 0.6);
  background: rgba(255, 255, 255, 0.04);
  border-radius: 0 6px 6px 0;
  color: rgba(255, 255, 255, 0.7);
}

.markdown-body :deep(table) {
  margin: 8px 0;
  border-collapse: collapse;
  font-size: 12px;
  max-width: 100%;
  display: block;
  overflow-x: auto;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  padding: 6px 12px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  text-align: left;
}

.markdown-body :deep(th) {
  background: rgba(255, 255, 255, 0.06);
  font-weight: 600;
}

.markdown-body :deep(code:not(.hljs)) {
  padding: 2px 6px;
  font-size: 12px;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 5px;
  color: #ffb86c;
}

.markdown-body :deep(.code-block) {
  margin: 8px 0;
  padding: 12px 14px;
  background: rgba(0, 0, 0, 0.45);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
  overflow-x: auto;
}

.markdown-body :deep(.code-block code) {
  font-size: 12px;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  line-height: 1.6;
  background: transparent;
  padding: 0;
  color: rgba(255, 255, 255, 0.88);
}

.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  margin: 12px 0;
}

/* 流式光标 */
.markdown-body.streaming::after {
  content: "▍";
  display: inline-block;
  color: #0a84ff;
  animation: blink 0.9s infinite;
  margin-left: 1px;
  vertical-align: baseline;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

/* 输入等待动画 */
.typing-indicator {
  display: flex;
  gap: 4px;
  padding: 10px 4px;
}

.typing-indicator span {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.4);
  animation: typing 1.2s infinite;
}

.typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
.typing-indicator span:nth-child(3) { animation-delay: 0.4s; }

@keyframes typing {
  0%, 60%, 100% { transform: translateY(0); opacity: 0.4; }
  30% { transform: translateY(-4px); opacity: 1; }
}

/* 消息元信息 */
.message-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 4px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
}

.message-item.user .message-meta {
  justify-content: flex-end;
}

.copy-btn {
  padding: 2px 8px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.5);
  background: transparent;
  border: none;
  border-radius: 5px;
  cursor: pointer;
  transition: all 120ms ease;
}

.copy-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.9);
}

.copy-btn.copied {
  color: #30d158;
}

/* 错误 + 重试 */
.message-error {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-top: 6px;
  padding: 8px 12px;
  background: rgba(255, 82, 82, 0.12);
  border: 1px solid rgba(255, 82, 82, 0.25);
  border-radius: 8px;
}

.error-text {
  flex: 1;
  font-size: 12px;
  color: #ff8787;
  word-break: break-all;
}

.retry-btn {
  padding: 4px 12px;
  font-size: 12px;
  color: #fff;
  background: #007aff;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  flex-shrink: 0;
}

.retry-btn:hover:not(:disabled) {
  background: #0069d9;
}

.retry-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 欢迎与空状态 */
.welcome,
.empty-state {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  text-align: center;
}

.welcome-icon,
.empty-icon {
  font-size: 44px;
  margin-bottom: 4px;
}

.welcome-title,
.empty-title {
  font-size: 15px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
}

.welcome-tips {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.tip {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.45);
}

.empty-desc {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
  line-height: 1.6;
  max-width: 320px;
}

.empty-desc b {
  color: rgba(255, 255, 255, 0.8);
}

/* ===== 输入区 ===== */
.chat-input-area {
  padding: 10px 16px 14px;
  background: rgba(40, 40, 40, 0.85);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.stats-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
}

.input-row {
  display: flex;
  gap: 10px;
  align-items: flex-end;
}

.chat-input {
  flex: 1;
  resize: none;
  padding: 9px 14px;
  font-size: 13px;
  font-family: inherit;
  line-height: 1.5;
  color: rgba(255, 255, 255, 0.95);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 10px;
  outline: none;
  max-height: 120px;
  transition: border-color 150ms ease;
}

.chat-input:focus {
  border-color: #007aff;
}

.chat-input::placeholder {
  color: rgba(255, 255, 255, 0.35);
}

.chat-input:disabled {
  opacity: 0.55;
}

.send-btn,
.stop-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 9px 16px;
  font-size: 13px;
  color: #fff;
  border: none;
  border-radius: 10px;
  cursor: pointer;
  flex-shrink: 0;
  transition: all 150ms ease;
}

.send-btn {
  background: linear-gradient(135deg, #0a84ff, #0066d6);
}

.send-btn:hover:not(:disabled) {
  filter: brightness(1.12);
}

.send-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.stop-btn {
  background: rgba(255, 82, 82, 0.85);
}

.stop-btn:hover {
  background: rgba(255, 82, 82, 1);
}

/* ===== 设置面板 ===== */
.settings-panel {
  position: absolute;
  top: 8px;
  right: 12px;
  z-index: 20;
  width: 280px;
  padding: 14px 16px;
  background: rgba(48, 48, 48, 0.97);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.45);
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  font-size: 13px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.95);
}

.close-panel-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  background: transparent;
  border: none;
  border-radius: 5px;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
}

.close-panel-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.9);
}

.settings-field {
  margin-bottom: 14px;
}

.field-label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.7);
}

.system-prompt-input {
  width: 100%;
  resize: vertical;
  padding: 8px 10px;
  font-size: 12px;
  font-family: inherit;
  line-height: 1.5;
  color: rgba(255, 255, 255, 0.9);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  outline: none;
  box-sizing: border-box;
}

.system-prompt-input:focus {
  border-color: #007aff;
}

.context-slider {
  width: 100%;
  accent-color: #007aff;
  cursor: pointer;
}

.slider-marks {
  display: flex;
  justify-content: space-between;
  margin-top: 2px;
  font-size: 10px;
  color: rgba(255, 255, 255, 0.35);
}

/* ===== 确认对话框 ===== */
.confirm-overlay {
  position: absolute;
  inset: 0;
  z-index: 30;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.4);
}

.confirm-dialog {
  width: 280px;
  padding: 18px;
  background: rgba(52, 52, 52, 0.98);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 14px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
}

.confirm-title {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.95);
  margin-bottom: 8px;
}

.confirm-desc {
  font-size: 12px;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.6);
  margin-bottom: 16px;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.cancel-btn {
  padding: 6px 14px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.85);
  background: rgba(255, 255, 255, 0.08);
  border: none;
  border-radius: 8px;
  cursor: pointer;
}

.cancel-btn:hover {
  background: rgba(255, 255, 255, 0.14);
}

.danger-btn {
  padding: 6px 14px;
  font-size: 12px;
  color: #fff;
  background: #ff5252;
  border: none;
  border-radius: 8px;
  cursor: pointer;
}

.danger-btn:hover {
  background: #ff3b3b;
}
</style>
