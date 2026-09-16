<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CalcLineResult } from "../types";

const props = defineProps<{
  initialExpression?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

// ===== 状态 =====

const lines = ref<string[]>([]);
const results = ref<CalcLineResult[]>([]);
const draft = ref("");
const preview = ref<CalcLineResult | null>(null);
const copiedValue = ref("");
const inputEl = ref<HTMLInputElement | null>(null);
const paperEl = ref<HTMLElement | null>(null);

// 输入历史（上/下键导航）
const inputHistory = ref<string[]>([]);
let historyIndex = -1;

let previewTimer: number | null = null;
let copiedTimer: number | null = null;

// ===== 求值 =====

async function reevaluate(paper: string[]) {
  try {
    results.value = await invoke<CalcLineResult[]>("evaluate_scratchpad", { lines: paper });
  } catch (e) {
    console.error("求值失败:", e);
    results.value = [];
  }
  nextTick(() => scrollToBottom());
}

async function updatePreview() {
  const text = draft.value.trim();
  if (!text) {
    preview.value = null;
    return;
  }
  try {
    // 带上整页上下文求值草稿行（支持引用前面的变量与 ans）
    const all = await invoke<CalcLineResult[]>("evaluate_scratchpad", {
      lines: [...lines.value, text],
    });
    preview.value = all[all.length - 1] || null;
  } catch (e) {
    console.error("预览失败:", e);
    preview.value = null;
  }
}

function schedulePreview() {
  if (previewTimer) clearTimeout(previewTimer);
  previewTimer = window.setTimeout(() => {
    previewTimer = null;
    updatePreview();
  }, 150);
}

// ===== 行操作 =====

async function commitDraft() {
  const text = draft.value.trim();
  if (!text) return;

  lines.value.push(text);
  inputHistory.value.push(text);
  historyIndex = -1;
  draft.value = "";
  preview.value = null;

  await persist();
  await reevaluate(lines.value);
  nextTick(() => inputEl.value?.focus());
}

async function deleteLine(index: number) {
  lines.value.splice(index, 1);
  await persist();
  await reevaluate(lines.value);
}

async function clearPaper() {
  if (!lines.value.length) return;
  if (!confirm("确定要清空整页稿纸吗？")) return;
  lines.value = [];
  results.value = [];
  inputHistory.value = [];
  historyIndex = -1;
  await persist();
  nextTick(() => inputEl.value?.focus());
}

async function persist() {
  try {
    await invoke("save_calc_lines", { lines: lines.value });
  } catch (e) {
    console.error("保存稿纸失败:", e);
  }
}

// ===== 交互 =====

function onInputKeydown(e: KeyboardEvent) {
  if (e.key === "Enter" && !e.shiftKey && !e.isComposing) {
    e.preventDefault();
    commitDraft();
  } else if (e.key === "ArrowUp") {
    // 草稿为空或已在历史中时，向上翻输入历史
    if (!draft.value || historyIndex !== -1) {
      if (inputHistory.value.length === 0) return;
      e.preventDefault();
      if (historyIndex === -1) {
        historyIndex = inputHistory.value.length - 1;
      } else if (historyIndex > 0) {
        historyIndex -= 1;
      }
      draft.value = inputHistory.value[historyIndex];
      schedulePreview();
    }
  } else if (e.key === "ArrowDown") {
    if (historyIndex !== -1) {
      e.preventDefault();
      if (historyIndex < inputHistory.value.length - 1) {
        historyIndex += 1;
        draft.value = inputHistory.value[historyIndex];
      } else {
        historyIndex = -1;
        draft.value = "";
      }
      schedulePreview();
    }
  }
}

function useExpression(text: string) {
  // 点击历史行表达式 → 载入输入框编辑
  draft.value = text;
  schedulePreview();
  nextTick(() => inputEl.value?.focus());
}

async function copyValue(value: string) {
  try {
    await invoke("write_clipboard_text", { text: value });
    copiedValue.value = value;
    if (copiedTimer) clearTimeout(copiedTimer);
    copiedTimer = window.setTimeout(() => {
      copiedValue.value = "";
    }, 1500);
  } catch (e) {
    console.error("复制失败:", e);
  }
}

function scrollToBottom() {
  const el = paperEl.value;
  if (el) el.scrollTop = el.scrollHeight;
}

// ===== 生命周期 =====

onMounted(async () => {
  try {
    const saved = await invoke<string[]>("load_calc_lines");
    lines.value = (saved || []).filter((l) => l.trim());
  } catch (e) {
    console.error("加载稿纸失败:", e);
  }
  await reevaluate(lines.value);

  if (props.initialExpression?.trim()) {
    draft.value = props.initialExpression.trim();
    updatePreview();
  }
  nextTick(() => inputEl.value?.focus());
});

onUnmounted(() => {
  if (previewTimer) clearTimeout(previewTimer);
  if (copiedTimer) clearTimeout(copiedTimer);
});
</script>

<template>
  <div class="calc-plugin">
    <!-- 顶栏 -->
    <div class="plugin-header" data-tauri-drag-region>
      <div class="header-left">
        <button class="back-btn" title="返回" @click="emit('close')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5" />
            <path d="M12 19l-7-7 7-7" />
          </svg>
        </button>
        <span class="plugin-title">计算稿纸</span>
      </div>
      <div class="header-actions">
        <button class="tool-btn" title="清空稿纸" @click="clearPaper">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 6h18" />
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
            <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          </svg>
        </button>
      </div>
    </div>

    <!-- 稿纸区 -->
    <div ref="paperEl" class="paper">
      <div v-if="lines.length === 0 && !draft" class="paper-empty">
        <div class="empty-icon">🧮</div>
        <div class="empty-title">在下方输入算式，Enter 记录一行</div>
        <div class="empty-desc">
          支持 + - * / % ^ () 与函数、常量；<code>a = 1+2</code> 定义变量，
          <code>ans</code> 引用上一行结果
        </div>
      </div>

      <div v-for="(line, index) in lines" :key="`${index}-${line}`" class="paper-line">
        <div class="line-input" title="点击载入编辑" @click="useExpression(line)">
          {{ line }}
        </div>
        <div class="line-right">
          <template v-if="results[index]?.ok">
            <span class="line-value" title="点击复制结果" @click="copyValue(results[index]!.value!)">
              <span v-if="results[index]!.is_assignment" class="var-badge">
                {{ results[index]!.var_name }} =
              </span>
              {{ results[index]!.value }}
            </span>
            <span v-if="copiedValue === results[index]!.value" class="copied-tag">已复制</span>
          </template>
          <span v-else class="line-error">{{ results[index]?.error || "求值失败" }}</span>
          <button class="line-delete" title="删除此行" @click="deleteLine(index)">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M18 6L6 18M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- 输入区 -->
    <div class="input-area">
      <div class="input-row">
        <input
          ref="inputEl"
          v-model="draft"
          type="text"
          class="calc-input"
          placeholder="输入算式，如 100 * 25% 或 a = 1 + 2"
          autocomplete="off"
          spellcheck="false"
          @keydown="onInputKeydown"
          @input="schedulePreview"
        />
        <div class="preview" :class="{ error: preview && !preview.ok }">
          <template v-if="preview && preview.ok">
            <span class="preview-eq">=</span>
            <span
              class="preview-value"
              title="点击复制"
              @click="copyValue(preview.value!)"
            >
              {{ preview.value }}
            </span>
          </template>
          <span v-else-if="preview && !preview.ok" class="preview-error">
            {{ preview.error }}
          </span>
          <span v-else class="preview-placeholder">=</span>
        </div>
        <button
          class="commit-btn"
          :disabled="!draft.trim()"
          title="记录（Enter）"
          @click="commitDraft"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 5v14M5 12h14" />
          </svg>
        </button>
      </div>
      <div class="hint-row">
        <span>↑↓ 输入历史</span>
        <span>函数 sin cos tan sqrt ln log abs round floor ceil min max pow</span>
        <span>常量 pi e</span>
        <span>三角函数用弧度</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.calc-plugin {
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

.tool-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.65);
  cursor: pointer;
}

.tool-btn:hover {
  background: rgba(255, 82, 82, 0.2);
  color: #ff8787;
}

/* ===== 稿纸 ===== */
.paper {
  flex: 1;
  overflow-y: auto;
  padding: 14px 20px;
}

.paper::-webkit-scrollbar {
  width: 8px;
}

.paper::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
}

.paper-empty {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  text-align: center;
}

.empty-icon {
  font-size: 40px;
}

.empty-title {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.7);
}

.empty-desc {
  font-size: 11px;
  line-height: 1.7;
  color: rgba(255, 255, 255, 0.35);
}

.empty-desc code {
  padding: 1px 5px;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 4px;
  color: #7ee2d8;
}

.paper-line {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 7px 4px;
  border-bottom: 1px dashed rgba(255, 255, 255, 0.08);
}

.line-input {
  flex: 1;
  min-width: 0;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  cursor: pointer;
  border-radius: 6px;
  padding: 2px 6px;
  margin-left: -6px;
}

.line-input:hover {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.95);
}

.line-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.line-value {
  font-family: "SF Mono", Menlo, Consolas, monospace;
  font-size: 14px;
  font-weight: 600;
  color: #4fe0c8;
  cursor: pointer;
  border-radius: 6px;
  padding: 2px 6px;
  transition: background 120ms ease;
}

.line-value:hover {
  background: rgba(79, 224, 200, 0.12);
}

.var-badge {
  font-size: 11px;
  font-weight: 400;
  color: rgba(255, 255, 255, 0.45);
}

.copied-tag {
  font-size: 10px;
  color: #30d158;
}

.line-error {
  font-size: 12px;
  color: #ff8787;
}

.line-delete {
  display: none;
  width: 20px;
  height: 20px;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 5px;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
}

.paper-line:hover .line-delete {
  display: flex;
}

.line-delete:hover {
  background: rgba(255, 82, 82, 0.2);
  color: #ff8787;
}

/* ===== 输入区 ===== */
.input-area {
  padding: 10px 16px 12px;
  background: rgba(40, 40, 40, 0.85);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.input-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.calc-input {
  flex: 1;
  padding: 10px 14px;
  font-size: 14px;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  color: rgba(255, 255, 255, 0.95);
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 10px;
  outline: none;
  transition: border-color 150ms ease;
}

.calc-input:focus {
  border-color: #30cfd0;
}

.calc-input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.preview {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 90px;
  justify-content: flex-end;
  font-family: "SF Mono", Menlo, Consolas, monospace;
}

.preview-eq,
.preview-placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.preview-value {
  font-size: 15px;
  font-weight: 600;
  color: #4fe0c8;
  cursor: pointer;
  border-radius: 6px;
  padding: 2px 6px;
}

.preview-value:hover {
  background: rgba(79, 224, 200, 0.12);
}

.preview.error .preview-error {
  font-size: 11px;
  color: #ff8787;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.commit-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #30cfd0, #268bd1);
  border: none;
  border-radius: 10px;
  color: #fff;
  cursor: pointer;
  flex-shrink: 0;
  transition: filter 150ms ease;
}

.commit-btn:hover:not(:disabled) {
  filter: brightness(1.12);
}

.commit-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.hint-row {
  display: flex;
  gap: 14px;
  margin-top: 8px;
  font-size: 10px;
  color: rgba(255, 255, 255, 0.3);
  flex-wrap: wrap;
}
</style>
