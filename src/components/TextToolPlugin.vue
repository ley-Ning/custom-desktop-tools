<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  initialInput?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

// ===== 模式与方向 =====

type Mode = "base64" | "url" | "hex" | "hash";
type Direction = "encode" | "decode";

const mode = ref<Mode>("base64");
const direction = ref<Direction>("encode");

const modes: Array<{ id: Mode; name: string }> = [
  { id: "base64", name: "Base64" },
  { id: "url", name: "URL" },
  { id: "hex", name: "Hex" },
  { id: "hash", name: "哈希" },
];

const directionLabel = computed(() => {
  if (mode.value === "hash") return "";
  return direction.value === "encode" ? "编码" : "解码";
});

// ===== 状态 =====

const inputText = ref("");
const outputText = ref("");
const errorText = ref("");
const copied = ref(false);

const hashResults = ref<Array<{ algo: string; value: string }>>([]);
const hashCopiedAlgo = ref("");

const inputEl = ref<HTMLTextAreaElement | null>(null);

const inputBytes = computed(
  () => new TextEncoder().encode(inputText.value).length
);

// Base64 特征：足够长的 base64 字母表字符（允许空白/填充），用于初始方向猜测
function looksLikeBase64(s: string): boolean {
  const compact = s.replace(/\s+/g, "");
  return compact.length >= 16 && /^[A-Za-z0-9+/]+={0,2}$/.test(compact);
}

// ===== 转换核心 =====

async function runTransform() {
  const text = inputText.value;
  errorText.value = "";
  outputText.value = "";

  if (mode.value === "hash") {
    await runHash(text);
    return;
  }

  if (!text) return;

  try {
    if (mode.value === "base64") {
      outputText.value =
        direction.value === "encode"
          ? await invoke<string>("text_base64_encode", { input: text })
          : await invoke<string>("text_base64_decode", { input: text });
    } else if (mode.value === "url") {
      outputText.value =
        direction.value === "encode"
          ? encodeURIComponent(text)
          : decodeURIComponent(text.replace(/\+/g, "%20"));
    } else {
      // hex
      outputText.value =
        direction.value === "encode" ? textToHex(text) : hexToText(text);
    }
  } catch (e) {
    errorText.value = String(e);
  }
}

function textToHex(text: string): string {
  const bytes = new TextEncoder().encode(text);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join(" ");
}

function hexToText(hex: string): string {
  const compact = hex.replace(/0x/gi, "").replace(/[\s,]/g, "");
  if (!/^[0-9a-fA-F]*$/.test(compact)) {
    throw new Error("无效的 Hex：包含非十六进制字符");
  }
  if (compact.length % 2 !== 0) {
    throw new Error("无效的 Hex：长度（去掉空白后）须为偶数");
  }
  const bytes = new Uint8Array(compact.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(compact.slice(i * 2, i * 2 + 2), 16);
  }
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new Error("解码结果不是有效的 UTF-8 文本");
  }
}

async function runHash(text: string) {
  hashResults.value = [];
  const algos = ["md5", "sha1", "sha256", "sha512"];
  try {
    const values = await Promise.all(
      algos.map((algo) => invoke<string>("text_hash", { input: text, algo }))
    );
    hashResults.value = algos.map((algo, i) => ({ algo, value: values[i] }));
  } catch (e) {
    errorText.value = String(e);
  }
}

// ===== 交互 =====

let transformTimer: ReturnType<typeof setTimeout> | null = null;

function scheduleTransform() {
  if (transformTimer) clearTimeout(transformTimer);
  transformTimer = setTimeout(runTransform, 250);
}

watch([mode, direction], () => {
  errorText.value = "";
  outputText.value = "";
  hashResults.value = [];
  scheduleTransform();
});

function switchDirection() {
  if (mode.value === "hash") return;
  direction.value =
    direction.value === "encode" ? "decode" : "encode";
  // 输出作为新的输入，继续反向操作
  if (outputText.value && !errorText.value) {
    inputText.value = outputText.value;
  }
  scheduleTransform();
}

function clearAll() {
  inputText.value = "";
  outputText.value = "";
  hashResults.value = [];
  errorText.value = "";
  nextTick(() => inputEl.value?.focus());
}

async function copyOutput() {
  if (!outputText.value) return;
  await invoke("write_clipboard_text", { text: outputText.value });
  copied.value = true;
  setTimeout(() => (copied.value = false), 1200);
}

async function copyHash(algo: string) {
  const row = hashResults.value.find((r) => r.algo === algo);
  if (!row) return;
  await invoke("write_clipboard_text", { text: row.value });
  hashCopiedAlgo.value = algo;
  setTimeout(() => (hashCopiedAlgo.value = ""), 1200);
}

const algoName: Record<string, string> = {
  md5: "MD5",
  sha1: "SHA-1",
  sha256: "SHA-256",
  sha512: "SHA-512",
};

// ===== 生命周期 =====

onMounted(() => {
  if (props.initialInput?.trim()) {
    inputText.value = props.initialInput;
    // 依据内容猜一个合理方向：像 Base64/Hex 就默认解码
    if (mode.value === "base64" && looksLikeBase64(props.initialInput)) {
      direction.value = "decode";
    }
  }
  runTransform();
  nextTick(() => inputEl.value?.focus());
});

onUnmounted(() => {
  if (transformTimer) clearTimeout(transformTimer);
});
</script>

<template>
  <div class="texttool-plugin">
    <!-- 顶栏 -->
    <div class="plugin-header" data-tauri-drag-region>
      <div class="header-left">
        <button class="back-btn" title="返回" @click="emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5" />
            <path d="M12 19l-7-7 7-7" />
          </svg>
        </button>
        <span class="plugin-title">文本工具箱</span>
      </div>

      <div class="header-actions">
        <div class="mode-tabs">
          <button
            v-for="m in modes"
            :key="m.id"
            class="mode-tab"
            :class="{ active: mode === m.id }"
            @click="mode = m.id"
          >
            {{ m.name }}
          </button>
        </div>
      </div>
    </div>

    <div class="texttool-body">
      <!-- 错误提示 -->
      <div v-if="errorText" class="error-bar">{{ errorText }}</div>

      <!-- 转换类模式：输入 / 输出 双卡片 -->
      <template v-if="mode !== 'hash'">
        <div class="text-card">
          <div class="card-toolbar">
            <span class="card-label">
              输入
              <span class="dir-badge">{{ directionLabel }}</span>
            </span>
            <div class="toolbar-right">
              <span class="byte-counter" title="UTF-8 字节数">{{ inputBytes }} B</span>
              <button class="tool-btn" title="清空" @click="clearAll">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M3 6h18" />
                  <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                  <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                </svg>
              </button>
            </div>
          </div>
          <textarea
            ref="inputEl"
            v-model="inputText"
            class="text-area"
            placeholder="输入文本，自动转换…"
            spellcheck="false"
            @input="scheduleTransform"
          ></textarea>
        </div>

        <div class="swap-row">
          <button class="swap-btn" title="交换方向（输出作为输入）" @click="switchDirection">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M17 2l4 4-4 4" />
              <path d="M21 6H7a4 4 0 0 0-4 4v1" />
              <path d="M7 22l-4-4 4-4" />
              <path d="M3 18h14a4 4 0 0 0 4-4v-1" />
            </svg>
            <span>{{ direction === "encode" ? "转为解码" : "转为编码" }}</span>
          </button>
        </div>

        <div class="text-card output-card">
          <div class="card-toolbar">
            <span class="card-label">输出</span>
            <div class="toolbar-right">
              <button
                class="tool-btn copy-btn"
                :class="{ ok: copied }"
                :disabled="!outputText"
                :title="copied ? '已复制' : '复制结果'"
                @click="copyOutput"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" />
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </svg>
                {{ copied ? "已复制" : "复制" }}
              </button>
            </div>
          </div>
          <textarea
            class="text-area output"
            :value="outputText"
            readonly
            placeholder="结果将显示在这里"
            spellcheck="false"
          ></textarea>
        </div>
      </template>

      <!-- 哈希模式：输入 + 四算法结果 -->
      <template v-else>
        <div class="text-card">
          <div class="card-toolbar">
            <span class="card-label">输入文本</span>
            <div class="toolbar-right">
              <span class="byte-counter" title="UTF-8 字节数">{{ inputBytes }} B</span>
              <button class="tool-btn" title="清空" @click="clearAll">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M3 6h18" />
                  <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                  <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                </svg>
              </button>
            </div>
          </div>
          <textarea
            ref="inputEl"
            v-model="inputText"
            class="text-area hash-input"
            placeholder="输入文本，自动计算哈希（空文本也有哈希值）…"
            spellcheck="false"
            @input="scheduleTransform"
          ></textarea>
        </div>

        <div class="hash-results">
          <div v-for="row in hashResults" :key="row.algo" class="hash-row">
            <span class="hash-algo">{{ algoName[row.algo] }}</span>
            <code class="hash-value" :title="row.value">{{ row.value }}</code>
            <button
              class="tool-btn copy-btn"
              :class="{ ok: hashCopiedAlgo === row.algo }"
              title="复制"
              @click="copyHash(row.algo)"
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="9" y="9" width="13" height="13" rx="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
              {{ hashCopiedAlgo === row.algo ? "已复制" : "复制" }}
            </button>
          </div>
          <div v-if="!hashResults.length && !errorText" class="hash-empty">
            输入文本后自动计算
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.texttool-plugin {
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

.mode-tabs {
  display: flex;
  gap: 4px;
  padding: 3px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 10px;
}

.mode-tab {
  padding: 5px 14px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  background: transparent;
  border: none;
  border-radius: 7px;
  cursor: pointer;
  transition: all 150ms ease;
}

.mode-tab:hover {
  color: rgba(255, 255, 255, 0.9);
}

.mode-tab.active {
  color: rgba(255, 255, 255, 0.95);
  background: rgba(255, 255, 255, 0.14);
  font-weight: 600;
}

/* ===== 主体 ===== */
.texttool-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 16px 20px;
  min-height: 0;
}

.error-bar {
  padding: 8px 12px;
  font-size: 12px;
  color: #ff8a8a;
  background: rgba(255, 100, 100, 0.1);
  border: 1px solid rgba(255, 100, 100, 0.25);
  border-radius: 10px;
}

.text-card {
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  overflow: hidden;
}

.text-card:first-of-type {
  flex: 2;
}

.output-card {
  flex: 3;
}

.card-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.card-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.55);
}

.dir-badge {
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 500;
  color: #7dc4ff;
  background: rgba(125, 196, 255, 0.12);
  border-radius: 6px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.byte-counter {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  font-variant-numeric: tabular-nums;
}

.tool-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 9px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.65);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  cursor: pointer;
  transition: all 150ms ease;
}

.tool-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.95);
}

.tool-btn:disabled {
  opacity: 0.4;
  cursor: default;
}

.copy-btn.ok {
  color: #6ee7a0;
  border-color: rgba(110, 231, 160, 0.35);
  background: rgba(110, 231, 160, 0.1);
}

.text-area {
  flex: 1;
  width: 100%;
  padding: 12px 14px;
  font-family: "SF Mono", "Menlo", "Consolas", monospace;
  font-size: 13px;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.92);
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  white-space: pre-wrap;
  word-break: break-all;
}

.text-area::placeholder {
  color: rgba(255, 255, 255, 0.25);
}

.text-area.output {
  color: #7dc4ff;
}

/* ===== 交换按钮 ===== */
.swap-row {
  display: flex;
  justify-content: center;
}

.swap-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 16px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.7);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 999px;
  cursor: pointer;
  transition: all 150ms ease;
}

.swap-btn:hover {
  background: rgba(255, 255, 255, 0.14);
  color: rgba(255, 255, 255, 0.95);
}

/* ===== 哈希模式 ===== */
.hash-input {
  min-height: 110px;
}

.hash-results {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.hash-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
}

.hash-algo {
  flex-shrink: 0;
  width: 72px;
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.55);
}

.hash-value {
  flex: 1;
  font-family: "SF Mono", "Menlo", "Consolas", monospace;
  font-size: 12px;
  color: #7dc4ff;
  word-break: break-all;
  line-height: 1.5;
}

.hash-empty {
  padding: 18px;
  text-align: center;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.3);
}
</style>
