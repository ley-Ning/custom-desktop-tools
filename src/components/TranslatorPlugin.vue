<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AiModel, StreamChunkEvent, StreamDoneEvent } from "../types";

const props = defineProps<{
  initialText?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

// ===== 语言选项 =====

interface LangOption {
  code: string;
  name: string;
}

const langOptions: LangOption[] = [
  { code: "zh-CN", name: "中文" },
  { code: "en", name: "英语" },
  { code: "ja", name: "日语" },
  { code: "ko", name: "韩语" },
  { code: "fr", name: "法语" },
  { code: "de", name: "德语" },
  { code: "es", name: "西班牙语" },
  { code: "ru", name: "俄语" },
  { code: "pt", name: "葡萄牙语" },
  { code: "it", name: "意大利语" },
  { code: "ar", name: "阿拉伯语" },
  { code: "th", name: "泰语" },
];

function langName(code: string): string {
  if (code === "auto") return "自动检测";
  return langOptions.find((l) => l.code === code)?.name || code;
}

// ===== 状态 =====

const sourceText = ref("");
const resultText = ref("");
const detectedLang = ref("");
const translating = ref(false);
const errorText = ref("");
const copied = ref(false);

const fromLang = ref("auto");
const toLang = ref("zh-CN");
const engine = ref<"mymemory" | "google" | "ai">("mymemory");

const models = ref<AiModel[]>([]);
const aiModelId = ref("");

// AI 流式请求标识（用于过滤 ai-message-chunk 事件）
let aiRequestId = "";

const MYMEMORY_LIMIT_BYTES = 500;

const sourceEl = ref<HTMLTextAreaElement | null>(null);

// 引擎列表（有可用模型时才提供 AI 翻译）
const engines = computed(() => {
  const list: Array<{ id: "mymemory" | "google" | "ai"; name: string }> = [
    { id: "mymemory", name: "MyMemory" },
    { id: "google", name: "Google" },
  ];
  if (models.value.length > 0) {
    list.push({ id: "ai", name: "AI 翻译" });
  }
  return list;
});

const sourceBytes = computed(() => new TextEncoder().encode(sourceText.value).length);
const overMyMemoryLimit = computed(
  () => engine.value === "mymemory" && sourceBytes.value > MYMEMORY_LIMIT_BYTES
);
const detectedLabel = computed(() => {
  if (fromLang.value !== "auto") return langName(fromLang.value);
  return detectedLang.value ? `检测：${langName(detectedLang.value)}` : "自动检测";
});

// 智能目标语言：中文 → 英文，其他 → 中文
function smartTarget(text: string): string {
  return /[\u4e00-\u9fa5]/.test(text) ? "en" : "zh-CN";
}

// ===== 翻译 =====

async function translate() {
  const text = sourceText.value.trim();
  if (!text || translating.value) return;

  if (toLang.value === "auto") {
    errorText.value = "目标语言不能是自动检测";
    return;
  }
  if (engine.value === "mymemory" && sourceBytes.value > MYMEMORY_LIMIT_BYTES) {
    errorText.value = `MyMemory 单次上限 ${MYMEMORY_LIMIT_BYTES} 字节（当前 ${sourceBytes.value}），请删减或切换 AI 翻译`;
    return;
  }

  errorText.value = "";
  translating.value = true;

  if (engine.value === "ai") {
    await translateWithAi(text);
  } else {
    await translateWithApi(text);
  }
}

async function translateWithApi(text: string) {
  try {
    const result = await invoke<{ text: string; detected?: string }>("translate_text", {
      text,
      from: fromLang.value,
      to: toLang.value,
      engine: engine.value,
    });
    resultText.value = result.text;
    detectedLang.value = result.detected || "";
  } catch (e) {
    errorText.value = String(e);
    resultText.value = "";
  } finally {
    translating.value = false;
  }
}

async function translateWithAi(text: string) {
  const model = models.value.find((m) => m.id === aiModelId.value);
  if (!model) {
    errorText.value = "请先在设置中配置并启用 AI 模型";
    translating.value = false;
    return;
  }

  // 清空旧结果，流式填充
  resultText.value = "";
  detectedLang.value = fromLang.value === "auto" ? "" : fromLang.value;
  aiRequestId = `translator-${Date.now()}`;

  const systemPrompt =
    `你是专业翻译引擎。将用户内容翻译成${langName(toLang.value)}。` +
    "只输出译文本身，不要任何解释、标注或引号。保留原文的格式与换行。若内容已经是目标语言，原样输出。";

  try {
    // 复用 AI 对话的流式通道，译文通过 ai-message-chunk 事件到达
    await invoke("send_ai_message", {
      modelId: model.id,
      messageId: aiRequestId,
      messages: [
        { role: "system", content: systemPrompt },
        { role: "user", content: text },
      ],
    });
  } catch (e) {
    errorText.value = String(e);
    translating.value = false;
    aiRequestId = "";
  }
  // 完成与错误统一在 ai-message-done 监听中收尾
}

function finalizeAiStream(status: "ok" | "error" | "stopped", error?: string) {
  if (status === "error") {
    errorText.value = error || "AI 翻译失败";
  }
  translating.value = false;
  aiRequestId = "";
}

// ===== 交互 =====

let translateTimer: number | null = null;

function scheduleTranslate(delay = 600) {
  if (translateTimer) clearTimeout(translateTimer);
  translateTimer = window.setTimeout(() => {
    translateTimer = null;
    translate();
  }, delay);
}

watch(sourceText, (text) => {
  errorText.value = "";
  if (!text.trim()) {
    resultText.value = "";
    detectedLang.value = "";
    if (translateTimer) clearTimeout(translateTimer);
    return;
  }
  // 源语言为自动时，跟随内容智能调整目标语言（仅当目标未被手动改过）
  if (fromLang.value === "auto" && !targetManuallySet) {
    toLang.value = smartTarget(text);
  }
  scheduleTranslate();
});

let targetManuallySet = false;

watch(toLang, () => {
  targetManuallySet = true;
});

watch(engine, () => {
  errorText.value = "";
  // 切换引擎后立即用当前内容重译
  if (sourceText.value.trim()) scheduleTranslate(100);
});

function onSourceKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
    e.preventDefault();
    if (translateTimer) clearTimeout(translateTimer);
    translate();
  }
}

function onPaste() {
  // 粘贴后立即翻译
  nextTick(() => scheduleTranslate(150));
}

function swapLanguages() {
  if (fromLang.value === "auto") {
    // 自动检测时：用检测到的语言（或目标语言）作为新源语言
    fromLang.value = detectedLang.value || toLang.value;
    toLang.value = /[\u4e00-\u9fa5]/.test(sourceText.value) ? "zh-CN" : smartTarget(detectedLang.value || toLang.value);
    if (toLang.value === fromLang.value) {
      toLang.value = fromLang.value === "zh-CN" ? "en" : "zh-CN";
    }
  } else {
    const from = fromLang.value;
    fromLang.value = toLang.value;
    toLang.value = from;
  }
  targetManuallySet = true;
  if (sourceText.value.trim()) {
    // 交换后把译文填回输入框
    if (resultText.value) {
      sourceText.value = resultText.value;
      resultText.value = "";
    }
    scheduleTranslate(100);
  }
}

function clearSource() {
  sourceText.value = "";
  resultText.value = "";
  detectedLang.value = "";
  errorText.value = "";
  nextTick(() => sourceEl.value?.focus());
}

async function copyResult() {
  if (!resultText.value) return;
  try {
    await invoke("write_clipboard_text", { text: resultText.value });
    copied.value = true;
    setTimeout(() => (copied.value = false), 1500);
  } catch (e) {
    console.error("复制失败:", e);
  }
}

// ===== AI 模型加载 =====

async function loadModels() {
  try {
    const configStr = await invoke<string>("load_ai_config");
    if (configStr) {
      const config = JSON.parse(configStr) as { models?: AiModel[] };
      models.value = (config.models || []).filter((m) => m.enabled);
    }
  } catch (e) {
    console.error("加载 AI 模型配置失败:", e);
  }
  if (models.value.length > 0) {
    aiModelId.value =
      (models.value.find((m) => m.isDefault) || models.value[0]).id;
  }
}

// ===== 事件监听 =====

let unlistenChunk: (() => void) | null = null;
let unlistenDone: (() => void) | null = null;

async function setupListeners() {
  unlistenChunk = await listen<StreamChunkEvent>("ai-message-chunk", (event) => {
    if (!aiRequestId || event.payload.message_id !== aiRequestId) return;
    resultText.value += event.payload.content;
  });

  unlistenDone = await listen<StreamDoneEvent>("ai-message-done", (event) => {
    if (!aiRequestId) return;
    const { message_id, status, error } = event.payload;
    if (message_id && message_id !== aiRequestId) return;
    finalizeAiStream(status, error);
  });
}

// ===== 生命周期 =====

onMounted(async () => {
  await loadModels();
  await setupListeners();

  if (props.initialText?.trim()) {
    sourceText.value = props.initialText;
    toLang.value = smartTarget(props.initialText);
  }
  nextTick(() => sourceEl.value?.focus());
});

onUnmounted(() => {
  if (translateTimer) clearTimeout(translateTimer);
  if (unlistenChunk) unlistenChunk();
  if (unlistenDone) unlistenDone();
  if (translating.value && aiRequestId) {
    invoke("stop_ai_generation").catch(() => {});
  }
});
</script>

<template>
  <div class="translator-plugin">
    <!-- 顶栏 -->
    <div class="plugin-header" data-tauri-drag-region>
      <div class="header-left">
        <button class="back-btn" title="返回" @click="emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5" />
            <path d="M12 19l-7-7 7-7" />
          </svg>
        </button>
        <span class="plugin-title">聚合翻译</span>
      </div>

      <div class="header-actions">
        <select v-model="engine" class="engine-select" title="翻译引擎">
          <option v-for="e in engines" :key="e.id" :value="e.id">{{ e.name }}</option>
        </select>
        <select
          v-if="engine === 'ai'"
          v-model="aiModelId"
          class="engine-select model-select"
          title="AI 模型"
        >
          <option v-for="model in models" :key="model.id" :value="model.id">
            {{ model.name }}
          </option>
        </select>
      </div>
    </div>

    <div class="translator-body">
      <!-- 源语言区 -->
      <div class="text-card source-card">
        <div class="card-toolbar">
          <select v-model="fromLang" class="lang-select">
            <option value="auto">自动检测</option>
            <option v-for="lang in langOptions" :key="lang.code" :value="lang.code">
              {{ lang.name }}
            </option>
          </select>
          <div class="toolbar-right">
            <span
              class="byte-counter"
              :class="{ over: overMyMemoryLimit }"
              :title="`UTF-8 字节数（MyMemory 上限 ${MYMEMORY_LIMIT_BYTES}）`"
            >
              {{ sourceBytes }} B
            </span>
            <button v-if="sourceText" class="tool-btn" title="清空" @click="clearSource">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
            <button
              class="translate-btn"
              :disabled="!sourceText.trim() || translating || overMyMemoryLimit"
              title="翻译（Enter）"
              @click="translate"
            >
              <svg v-if="translating" class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12a9 9 0 1 1-6.219-8.56" />
              </svg>
              <span>{{ translating ? "翻译中" : "翻译" }}</span>
            </button>
          </div>
        </div>
        <textarea
          ref="sourceEl"
          v-model="sourceText"
          class="source-textarea"
          placeholder="输入或粘贴要翻译的内容，自动翻译；Enter 立即翻译，Shift + Enter 换行"
          @keydown="onSourceKeydown"
          @paste="onPaste"
        ></textarea>
      </div>

      <!-- 交换按钮 -->
      <div class="swap-row">
        <button class="swap-btn" title="交换语言（译文填回输入框）" @click="swapLanguages">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M17 1l4 4-4 4" />
            <path d="M3 11V9a4 4 0 0 1 4-4h14" />
            <path d="M7 23l-4-4 4-4" />
            <path d="M21 13v2a4 4 0 0 1-4 4H3" />
          </svg>
        </button>
      </div>

      <!-- 译文区 -->
      <div class="text-card result-card">
        <div class="card-toolbar">
          <select v-model="toLang" class="lang-select">
            <option v-for="lang in langOptions" :key="lang.code" :value="lang.code">
              {{ lang.name }}
            </option>
          </select>
          <div class="toolbar-right">
            <span v-if="detectedLabel" class="detected-tag">{{ detectedLabel }}</span>
            <button
              v-if="resultText"
              class="copy-btn"
              :class="{ copied }"
              @click="copyResult"
            >
              {{ copied ? "已复制" : "复制" }}
            </button>
          </div>
        </div>

        <div class="result-area">
          <!-- 错误提示 -->
          <div v-if="errorText" class="error-banner">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10" />
              <path d="M12 8v4M12 16h.01" />
            </svg>
            <span>{{ errorText }}</span>
          </div>

          <div v-else-if="resultText" class="result-text" :class="{ streaming: translating && engine === 'ai' }">
            {{ resultText }}
          </div>

          <div v-else-if="!translating" class="result-empty">
            <div class="empty-icon">🌐</div>
            <div class="empty-title">译文将显示在这里</div>
            <div class="empty-desc">支持 MyMemory / Google / AI 多引擎，输入后自动翻译</div>
          </div>

          <div v-else class="result-empty">
            <div class="typing-indicator"><span></span><span></span><span></span></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.translator-plugin {
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
.plugin-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 12px 16px;
  background: rgba(40, 40, 40, 0.9);
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.back-btn {
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
  transition: all 150ms ease;
}

.back-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.95);
}

.plugin-title {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.95);
}

.header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.engine-select {
  padding: 5px 10px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.9);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 8px;
  outline: none;
  cursor: pointer;
}

.engine-select option {
  background: #2a2a2a;
  color: rgba(255, 255, 255, 0.9);
}

.model-select {
  max-width: 150px;
}

/* ===== 主体 ===== */
.translator-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0;
  padding: 16px 20px;
  min-height: 0;
}

.text-card {
  display: flex;
  flex-direction: column;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  overflow: hidden;
}

.source-card {
  flex: 1;
  min-height: 0;
}

.result-card {
  flex: 1;
  min-height: 0;
}

.card-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.03);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.lang-select {
  padding: 4px 8px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.9);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 7px;
  outline: none;
  cursor: pointer;
}

.lang-select option {
  background: #2a2a2a;
  color: rgba(255, 255, 255, 0.9);
}

.byte-counter {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
}

.byte-counter.over {
  color: #ff8787;
}

.tool-btn {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.9);
}

.translate-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 14px;
  font-size: 12px;
  color: #fff;
  background: linear-gradient(135deg, #0a84ff, #0066d6);
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: filter 150ms ease;
}

.translate-btn:hover:not(:disabled) {
  filter: brightness(1.12);
}

.translate-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.spin {
  animation: spin 0.9s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.source-textarea {
  flex: 1;
  resize: none;
  padding: 14px 16px;
  font-size: 14px;
  font-family: inherit;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.95);
  background: transparent;
  border: none;
  outline: none;
}

.source-textarea::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

/* ===== 交换 ===== */
.swap-row {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px 0;
}

.swap-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  color: rgba(255, 255, 255, 0.65);
  cursor: pointer;
  transition: all 150ms ease;
}

.swap-btn:hover {
  background: #007aff;
  border-color: #007aff;
  color: #fff;
  transform: rotate(180deg);
}

/* ===== 译文 ===== */
.result-area {
  flex: 1;
  overflow-y: auto;
  padding: 14px 16px;
}

.result-text {
  font-size: 14px;
  line-height: 1.65;
  color: rgba(255, 255, 255, 0.95);
  white-space: pre-wrap;
  word-break: break-word;
}

.result-text.streaming::after {
  content: "▍";
  color: #0a84ff;
  animation: blink 0.9s infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

.detected-tag {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.5);
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 8px;
}

.copy-btn {
  padding: 3px 10px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.6);
  background: rgba(255, 255, 255, 0.06);
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.copy-btn:hover {
  color: rgba(255, 255, 255, 0.95);
  background: rgba(255, 255, 255, 0.12);
}

.copy-btn.copied {
  color: #30d158;
}

/* 错误横幅 */
.error-banner {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 14px;
  background: rgba(255, 82, 82, 0.12);
  border: 1px solid rgba(255, 82, 82, 0.25);
  border-radius: 10px;
  color: #ff8787;
  font-size: 12px;
  line-height: 1.5;
  word-break: break-all;
}

.error-banner svg {
  flex-shrink: 0;
  margin-top: 1px;
}

/* 空状态 */
.result-empty {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.empty-icon {
  font-size: 36px;
  margin-bottom: 2px;
}

.empty-title {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.7);
}

.empty-desc {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
}

.typing-indicator {
  display: flex;
  gap: 4px;
  padding: 8px 0;
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
</style>
