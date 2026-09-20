<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import {
  createJSONEditor,
  isContentValidationErrors,
  isJSONContent,
  isTextContent,
  Mode,
  type Content,
  type JsonEditor,
  type OnChangeStatus,
} from "vanilla-jsoneditor";
import "vanilla-jsoneditor/themes/jse-theme-dark.css";

const emit = defineEmits<{
  close: [];
}>();

const editorContainer = ref<HTMLDivElement | null>(null);
const searchPath = ref("");
const searchResult = ref("");
const errorMessage = ref("");
const editorMode = ref<Mode>(Mode.tree);

// 底部状态栏
const statsValid = ref(true);
const statsSize = ref("0 B");
const statsNodes = ref(0);
const statsLines = ref(0);

const copied = ref(false);
let copiedTimer: number | null = null;

let editor: JsonEditor | null = null;
let statsTimer: number | null = null;

const SAMPLE = {
  名称: "custom-desktop-tools",
  功能: ["AI 对话", "聚合翻译", "计算稿纸"],
  配置: { 版本: "0.2.0", 平台: "macOS", 开源: true },
};

// ===== 内容读写 =====

function getText(): string {
  if (!editor) return "";
  const content = editor.get();
  if (isTextContent(content) && content.text) return content.text;
  if (isJSONContent(content) && content.json !== undefined) {
    return JSON.stringify(content.json, null, 2);
  }
  return "";
}

function setText(text: string) {
  editor?.set({ text });
}

function parseOrThrow(): unknown {
  try {
    return JSON.parse(getText());
  } catch {
    throw new Error("当前内容不是合法的 JSON，请先修复格式错误（可用文本模式编辑）");
  }
}

// ===== 工具操作 =====

function formatJson() {
  try {
    const obj = parseOrThrow();
    setText(JSON.stringify(obj, null, 2));
    errorMessage.value = "";
  } catch (e: any) {
    errorMessage.value = e.message;
  }
}

function minifyJson() {
  try {
    const obj = parseOrThrow();
    setText(JSON.stringify(obj));
    errorMessage.value = "";
  } catch (e: any) {
    errorMessage.value = e.message;
  }
}

// 转义：把当前文本变成单行转义字符串（去掉外层引号的字面量）
function escapeText() {
  const text = getText();
  if (!text) return;
  setText(JSON.stringify(text).slice(1, -1));
  errorMessage.value = "";
}

// 去转义：把转义字符串还原为真实文本
function unescapeText() {
  const text = getText().trim();
  if (!text) return;
  try {
    const decoded = JSON.parse(`"${text.replace(/^"|"$/g, "")}"`);
    setText(typeof decoded === "string" ? decoded : JSON.stringify(decoded, null, 2));
    errorMessage.value = "";
  } catch {
    errorMessage.value = "不是合法的转义字符串";
  }
}

// Unicode 转义：非 ASCII 字符转成 \uXXXX（压缩为一行）
function toUnicodeEscape() {
  try {
    const obj = parseOrThrow();
    const text = JSON.stringify(obj).replace(
      /[^\x20-\x7E]/g,
      (ch) => "\\u" + ch.charCodeAt(0).toString(16).padStart(4, "0")
    );
    setText(text);
    errorMessage.value = "";
  } catch (e: any) {
    errorMessage.value = e.message;
  }
}

// Unicode 还原：\uXXXX 由 JSON.parse 自动解码
function fromUnicodeEscape() {
  try {
    const obj = parseOrThrow();
    setText(JSON.stringify(obj, null, 2));
    errorMessage.value = "";
  } catch (e: any) {
    errorMessage.value = e.message;
  }
}

// 递归按键名排序（便于 diff / 对比）
function sortValue(value: any): any {
  if (Array.isArray(value)) return value.map(sortValue);
  if (value && typeof value === "object") {
    return Object.keys(value)
      .sort()
      .reduce<Record<string, any>>((acc, key) => {
        acc[key] = sortValue(value[key]);
        return acc;
      }, {});
  }
  return value;
}

function sortKeys() {
  try {
    const obj = parseOrThrow();
    setText(JSON.stringify(sortValue(obj), null, 2));
    errorMessage.value = "";
  } catch (e: any) {
    errorMessage.value = e.message;
  }
}

async function pasteFromClipboard() {
  try {
    const text = await navigator.clipboard.readText();
    if (text) {
      setText(text);
      errorMessage.value = "";
    }
  } catch (e: any) {
    errorMessage.value = `粘贴失败: ${e.message}`;
  }
}

async function copyToClipboard() {
  const text = getText();
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    copied.value = true;
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedTimer = window.setTimeout(() => (copied.value = false), 1500);
  } catch (e: any) {
    errorMessage.value = `复制失败: ${e.message}`;
  }
}

function clearEditor() {
  if (!getText()) return;
  if (confirm("确定要清空编辑器吗？")) {
    setText("");
    searchPath.value = "";
    searchResult.value = "";
    errorMessage.value = "";
  }
}

function switchMode(mode: Mode) {
  editorMode.value = mode;
  editor?.updateProps({ mode });
}

// ===== 路径搜索 =====

function searchByPath() {
  if (!editor || !searchPath.value.trim()) {
    searchResult.value = "";
    return;
  }

  try {
    const content = editor.get();
    const json = isJSONContent(content) ? content.json : null;
    if (!json) {
      searchResult.value = "请先输入有效的 JSON";
      return;
    }

    const path = searchPath.value.trim();
    const keys = path.split(".").filter((k) => k);

    let result: any = json;
    for (const key of keys) {
      if (key.includes("[") && key.includes("]")) {
        const arrayKey = key.substring(0, key.indexOf("["));
        const index = parseInt(key.substring(key.indexOf("[") + 1, key.indexOf("]")));
        if (arrayKey) result = result[arrayKey];
        if (Array.isArray(result) && !isNaN(index)) {
          result = result[index];
        } else {
          searchResult.value = `路径 "${path}" 不是有效的数组索引`;
          return;
        }
      } else if (result && typeof result === "object" && key in result) {
        result = result[key];
      } else {
        searchResult.value = `路径 "${path}" 不存在`;
        return;
      }
    }

    searchResult.value = JSON.stringify(result, null, 2);
  } catch (e: any) {
    searchResult.value = `搜索失败: ${e.message}`;
  }
}

async function copyResult() {
  if (!searchResult.value) return;
  try {
    await navigator.clipboard.writeText(searchResult.value);
    copied.value = true;
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedTimer = window.setTimeout(() => (copied.value = false), 1500);
  } catch {
    // 忽略
  }
}

// ===== 统计 =====

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function countNodes(value: any): number {
  if (Array.isArray(value)) return value.reduce((sum, v) => sum + countNodes(v), 1);
  if (value && typeof value === "object") {
    return Object.values(value).reduce((sum: number, v) => sum + countNodes(v), 1);
  }
  return 1;
}

function updateStats() {
  const text = getText();
  statsSize.value = formatBytes(new TextEncoder().encode(text).length);
  statsLines.value = text ? text.split("\n").length : 0;
  try {
    statsNodes.value = countNodes(JSON.parse(text));
    statsValid.value = true;
  } catch {
    statsNodes.value = 0;
    statsValid.value = false;
  }
}

function scheduleStats() {
  if (statsTimer) clearTimeout(statsTimer);
  statsTimer = window.setTimeout(() => {
    statsTimer = null;
    updateStats();
  }, 250);
}

// ===== 生命周期 =====

onMounted(() => {
  if (!editorContainer.value) return;
  editor = createJSONEditor({
    target: editorContainer.value,
    props: {
      mode: Mode.tree,
      mainMenuBar: false,
      navigationBar: true,
      statusBar: false,
      readOnly: false,
      content: { json: SAMPLE },
      onChange: (_content: Content, _previousContent: Content, status: OnChangeStatus) => {
        if (status.contentErrors && isContentValidationErrors(status.contentErrors)) {
          errorMessage.value = status.contentErrors.validationErrors[0]?.message || "JSON 格式错误";
        } else if (status.contentErrors) {
          errorMessage.value = "JSON 格式错误";
        } else {
          errorMessage.value = "";
        }
        scheduleStats();
      },
    },
  });
  updateStats();
});

onUnmounted(() => {
  if (editor) {
    editor.destroy();
    editor = null;
  }
  if (statsTimer) clearTimeout(statsTimer);
  if (copiedTimer) clearTimeout(copiedTimer);
});
</script>

<template>
  <div class="json-editor-plugin">
    <!-- 顶栏 -->
    <div class="plugin-header" data-tauri-drag-region>
      <div class="header-left">
        <button class="back-btn" title="返回" @click="emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5" />
            <path d="M12 19l-7-7 7-7" />
          </svg>
        </button>
        <span class="plugin-title">JSON 编辑器</span>

        <!-- 模式切换 -->
        <div class="mode-switch">
          <button
            class="mode-btn"
            :class="{ active: editorMode === Mode.tree }"
            @click="switchMode(Mode.tree)"
          >
            树形
          </button>
          <button
            class="mode-btn"
            :class="{ active: editorMode === Mode.text }"
            @click="switchMode(Mode.text)"
          >
            文本
          </button>
        </div>
      </div>

      <div class="header-actions">
        <button class="action-btn" title="从剪贴板粘贴" @click="pasteFromClipboard">粘贴</button>
        <button class="action-btn" :class="{ copied }" title="复制全部内容" @click="copyToClipboard">
          {{ copied ? "已复制" : "复制" }}
        </button>
        <button class="action-btn danger" title="清空" @click="clearEditor">清空</button>
      </div>
    </div>

    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-group">
        <button class="tool-btn primary" title="美化格式（2 空格缩进）" @click="formatJson">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="16 18 22 12 16 6" />
            <polyline points="8 6 2 12 8 18" />
          </svg>
          格式化
        </button>
        <button class="tool-btn primary" title="压缩为一行" @click="minifyJson">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M4 9h16M4 15h16" />
          </svg>
          压缩
        </button>
        <button class="tool-btn" title="递归按键名排序（便于对比）" @click="sortKeys">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M11 5h10M11 9h7M11 15h10M11 19h7" />
            <path d="M3 8l3 3 3-3" fill="none" />
            <path d="M3 16l3-3 3 3" fill="none" />
          </svg>
          排序键
        </button>
      </div>

      <div class="toolbar-sep"></div>

      <div class="toolbar-group">
        <button class="tool-btn" title="把当前文本转成单行转义字符串" @click="escapeText">转义</button>
        <button class="tool-btn" title="把转义字符串还原" @click="unescapeText">去转义</button>
        <button class="tool-btn" title="非 ASCII 字符转 \\uXXXX" @click="toUnicodeEscape">\u 转义</button>
        <button class="tool-btn" title="\\uXXXX 还原为原文" @click="fromUnicodeEscape">\u 还原</button>
      </div>

      <div class="toolbar-sep"></div>

      <!-- 路径搜索 -->
      <div class="path-search">
        <input
          v-model="searchPath"
          type="text"
          class="path-input"
          placeholder="路径取值，如 config.版本 或 功能[0]"
          spellcheck="false"
          @keyup.enter="searchByPath"
        />
        <button v-if="searchResult" class="copy-mini" title="复制结果" @click="copyResult">复制</button>
      </div>
    </div>

    <!-- 搜索结果 / 错误（叠加层，可关闭） -->
    <div v-if="searchResult" class="result-bar">
      <span class="result-label">路径结果</span>
      <pre class="result-content">{{ searchResult }}</pre>
      <button class="close-mini" title="关闭" @click="searchResult = ''">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12" />
        </svg>
      </button>
    </div>

    <div v-if="errorMessage" class="error-banner">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10" />
        <path d="M12 8v4M12 16h.01" />
      </svg>
      <span>{{ errorMessage }}</span>
    </div>

    <!-- 编辑器（官方暗色主题） -->
    <div ref="editorContainer" class="editor-container jse-theme-dark"></div>

    <!-- 状态栏 -->
    <div class="status-bar">
      <span class="status-item" :class="statsValid ? 'ok' : 'bad'">
        {{ statsValid ? "✓ JSON 有效" : "✗ 格式错误" }}
      </span>
      <span class="status-item">{{ statsSize }}</span>
      <span class="status-item">{{ statsNodes }} 节点</span>
      <span class="status-item">{{ statsLines }} 行</span>
      <span class="status-hint">Ctrl+F 搜索 · Ctrl+Z 撤销 · 树形模式可双击编辑</span>
    </div>
  </div>
</template>

<style scoped>
.json-editor-plugin {
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
  gap: 12px;
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

.mode-switch {
  display: flex;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  overflow: hidden;
}

.mode-btn {
  padding: 5px 12px;
  font-size: 12px;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.55);
  cursor: pointer;
  transition: all 150ms ease;
}

.mode-btn.active {
  background: #0a84ff;
  color: #fff;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 6px 12px;
  font-size: 12px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.8);
  cursor: pointer;
  transition: all 150ms ease;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #fff;
}

.action-btn.copied {
  color: #30d158;
}

.action-btn.danger:hover {
  background: rgba(255, 82, 82, 0.2);
  border-color: rgba(255, 82, 82, 0.4);
  color: #ff8787;
}

/* ===== 工具栏 ===== */
.toolbar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding: 8px 16px;
  background: rgba(36, 36, 36, 0.85);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.toolbar-group {
  display: flex;
  gap: 6px;
}

.toolbar-sep {
  width: 1px;
  height: 20px;
  background: rgba(255, 255, 255, 0.1);
}

.tool-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 10px;
  font-size: 12px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  color: rgba(255, 255, 255, 0.75);
  cursor: pointer;
  white-space: nowrap;
  transition: all 150ms ease;
}

.tool-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.tool-btn.primary {
  background: rgba(10, 132, 255, 0.15);
  border-color: rgba(10, 132, 255, 0.3);
  color: #5eb5ff;
}

.tool-btn.primary:hover {
  background: rgba(10, 132, 255, 0.28);
  color: #fff;
}

/* 路径搜索 */
.path-search {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 200px;
  margin-left: auto;
}

.path-input {
  flex: 1;
  padding: 5px 10px;
  font-size: 12px;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  color: rgba(255, 255, 255, 0.9);
  outline: none;
  transition: border-color 150ms ease;
}

.path-input:focus {
  border-color: #0a84ff;
}

.path-input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.copy-mini {
  padding: 4px 10px;
  font-size: 11px;
  background: rgba(255, 255, 255, 0.08);
  border: none;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.75);
  cursor: pointer;
  white-space: nowrap;
}

.copy-mini:hover {
  color: #fff;
  background: rgba(255, 255, 255, 0.14);
}

/* ===== 结果条 ===== */
.result-bar {
  position: relative;
  display: flex;
  align-items: stretch;
  gap: 12px;
  padding: 8px 12px;
  margin: 8px 16px 0;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
}

.result-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.45);
  white-space: nowrap;
  align-self: center;
}

.result-content {
  flex: 1;
  max-height: 120px;
  margin: 0;
  overflow: auto;
  font-size: 12px;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  color: #4fe0c8;
  white-space: pre-wrap;
  word-break: break-all;
}

.close-mini {
  align-self: flex-start;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: transparent;
  border: none;
  border-radius: 5px;
  color: rgba(255, 255, 255, 0.45);
  cursor: pointer;
}

.close-mini:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

/* ===== 错误横幅 ===== */
.error-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 8px 16px 0;
  padding: 8px 12px;
  background: rgba(255, 82, 82, 0.12);
  border: 1px solid rgba(255, 82, 82, 0.25);
  border-radius: 10px;
  color: #ff8787;
  font-size: 12px;
}

/* ===== 编辑器 ===== */
.editor-container {
  flex: 1;
  min-height: 0;
  margin-top: 8px;
  overflow: hidden;
}

/* jse 暗色主题与整体风格融合的微调 */
.editor-container :deep(.jse-main) {
  border-radius: 0 0 18px 18px;
}

/* ===== 状态栏 ===== */
.status-bar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 6px 16px;
  background: rgba(40, 40, 40, 0.9);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 11px;
  color: rgba(255, 255, 255, 0.5);
}

.status-item.ok {
  color: #30d158;
}

.status-item.bad {
  color: #ff8787;
}

.status-hint {
  margin-left: auto;
  color: rgba(255, 255, 255, 0.28);
}
</style>
