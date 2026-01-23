<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { EditorView, basicSetup } from "codemirror";
import { json, jsonParseLinter } from "@codemirror/lang-json";
import { linter } from "@codemirror/lint";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{
  close: [];
}>();

const editorContainer = ref<HTMLElement | null>(null);
const jsonInput = ref("");
const jsonOutput = ref("");
const filterPath = ref("");
const errorMessage = ref("");
const successMessage = ref("");
const activeTab = ref<"input" | "output">("input");

let editorView: EditorView | null = null;

// 在新窗口打开
async function openInNewWindow() {
  try {
    await invoke("open_plugin_window", { pluginId: "json" });
    successMessage.value = "已在新窗口打开";
    setTimeout(() => {
      successMessage.value = "";
    }, 2000);
  } catch (e) {
    errorMessage.value = "打开新窗口失败：" + e;
  }
}

// 初始化编辑器
onMounted(() => {
  if (editorContainer.value) {
    editorView = new EditorView({
      doc: jsonInput.value,
      extensions: [
        basicSetup,
        json(),
        linter(jsonParseLinter()),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            jsonInput.value = update.state.doc.toString();
          }
        }),
      ],
      parent: editorContainer.value,
    });
  }
});

onUnmounted(() => {
  editorView?.destroy();
});

// 格式化 JSON
function formatJson() {
  try {
    errorMessage.value = "";
    const parsed = JSON.parse(jsonInput.value);
    const formatted = JSON.stringify(parsed, null, 2);
    
    if (editorView) {
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: formatted,
        },
      });
    }
    
    jsonInput.value = formatted;
    successMessage.value = "格式化成功";
    setTimeout(() => {
      successMessage.value = "";
    }, 2000);
  } catch (e) {
    errorMessage.value = "JSON 格式错误：" + (e as Error).message;
  }
}

// 压缩 JSON
function minifyJson() {
  try {
    errorMessage.value = "";
    const parsed = JSON.parse(jsonInput.value);
    const minified = JSON.stringify(parsed);
    
    if (editorView) {
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: minified,
        },
      });
    }
    
    jsonInput.value = minified;
    successMessage.value = "压缩成功";
    setTimeout(() => {
      successMessage.value = "";
    }, 2000);
  } catch (e) {
    errorMessage.value = "JSON 格式错误：" + (e as Error).message;
  }
}

// 验证 JSON
function validateJson() {
  try {
    errorMessage.value = "";
    JSON.parse(jsonInput.value);
    successMessage.value = "✓ JSON 格式正确";
    setTimeout(() => {
      successMessage.value = "";
    }, 2000);
  } catch (e) {
    errorMessage.value = "JSON 格式错误：" + (e as Error).message;
  }
}

// 路径筛选
function filterByPath() {
  try {
    errorMessage.value = "";
    
    if (!filterPath.value.trim()) {
      errorMessage.value = "请输入路径，例如：user.name 或 items[0].id";
      return;
    }
    
    const parsed = JSON.parse(jsonInput.value);
    const result = getValueByPath(parsed, filterPath.value);
    
    if (result === undefined) {
      errorMessage.value = "路径不存在：" + filterPath.value;
      return;
    }
    
    jsonOutput.value = JSON.stringify(result, null, 2);
    activeTab.value = "output";
    successMessage.value = "筛选成功";
    setTimeout(() => {
      successMessage.value = "";
    }, 2000);
  } catch (e) {
    errorMessage.value = "筛选失败：" + (e as Error).message;
  }
}

// 根据路径获取值
function getValueByPath(obj: any, path: string): any {
  const keys = path.split(/\.|\[|\]/).filter(k => k);
  let current = obj;
  
  for (const key of keys) {
    if (current === null || current === undefined) {
      return undefined;
    }
    current = current[key];
  }
  
  return current;
}

// 清空输入
function clearInput() {
  if (confirm("确定要清空输入吗？")) {
    if (editorView) {
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: "",
        },
      });
    }
    jsonInput.value = "";
    jsonOutput.value = "";
    filterPath.value = "";
    errorMessage.value = "";
  }
}

// 复制到剪贴板
async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    successMessage.value = "已复制到剪贴板";
    setTimeout(() => {
      successMessage.value = "";
    }, 2000);
  } catch (e) {
    errorMessage.value = "复制失败";
  }
}

// 示例 JSON
function loadExample() {
  const example = {
    user: {
      id: 1,
      name: "张三",
      email: "zhangsan@example.com",
      profile: {
        age: 28,
        city: "北京",
        hobbies: ["阅读", "旅游", "摄影"]
      }
    },
    posts: [
      {
        id: 101,
        title: "第一篇文章",
        content: "这是内容...",
        tags: ["技术", "编程"]
      },
      {
        id: 102,
        title: "第二篇文章",
        content: "更多内容...",
        tags: ["生活", "随笔"]
      }
    ],
    settings: {
      theme: "dark",
      language: "zh-CN",
      notifications: true
    }
  };
  
  const formatted = JSON.stringify(example, null, 2);
  
  if (editorView) {
    editorView.dispatch({
      changes: {
        from: 0,
        to: editorView.state.doc.length,
        insert: formatted,
      },
    });
  }
  
  jsonInput.value = formatted;
}
</script>

<template>
  <div class="json-editor-plugin">
    <!-- 顶部工具栏 -->
    <div class="editor-header">
      <div class="header-left">
        <h2 class="editor-title">📊 JSON 编辑器</h2>
        <span class="editor-subtitle">格式化、验证、路径筛选</span>
      </div>
      <div class="header-actions">
        <button class="btn-new-window" @click="openInNewWindow" title="在新窗口打开">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
            <polyline points="15 3 21 3 21 9"/>
            <line x1="10" y1="14" x2="21" y2="3"/>
          </svg>
        </button>
        <button class="btn-close" @click="emit('close')">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-group">
        <button class="btn-tool" @click="formatJson" title="格式化 JSON">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="16 18 22 12 16 6"/>
            <polyline points="8 6 2 12 8 18"/>
          </svg>
          格式化
        </button>
        <button class="btn-tool" @click="minifyJson" title="压缩 JSON">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          压缩
        </button>
        <button class="btn-tool" @click="validateJson" title="验证 JSON">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"/>
          </svg>
          验证
        </button>
      </div>
      
      <div class="toolbar-group">
        <button class="btn-tool" @click="loadExample" title="加载示例">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
          </svg>
          示例
        </button>
        <button class="btn-tool" @click="clearInput" title="清空">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          清空
        </button>
      </div>
    </div>

    <!-- 路径筛选 -->
    <div class="filter-bar">
      <div class="filter-input-group">
        <svg class="filter-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
        </svg>
        <input
          v-model="filterPath"
          type="text"
          class="filter-input"
          placeholder="输入路径筛选，例如：user.name 或 posts[0].title"
          @keyup.enter="filterByPath"
        />
        <button class="btn-filter" @click="filterByPath">筛选</button>
      </div>
      <div class="filter-hint">
        💡 支持：<code>user.name</code>、<code>items[0]</code>、<code>data.list[1].id</code>
      </div>
    </div>

    <!-- 消息提示 -->
    <div v-if="errorMessage" class="message error-message">{{ errorMessage }}</div>
    <div v-if="successMessage" class="message success-message">{{ successMessage }}</div>

    <!-- 编辑器区域 -->
    <div class="editor-container">
      <!-- 标签页 -->
      <div class="tabs">
        <button
          class="tab"
          :class="{ active: activeTab === 'input' }"
          @click="activeTab = 'input'"
        >
          输入
        </button>
        <button
          class="tab"
          :class="{ active: activeTab === 'output' }"
          @click="activeTab = 'output'"
        >
          输出
          <span v-if="jsonOutput" class="tab-badge">•</span>
        </button>
      </div>

      <!-- 输入编辑器 -->
      <div v-show="activeTab === 'input'" class="editor-panel">
        <div class="editor-actions">
          <button class="btn-action" @click="copyToClipboard(jsonInput)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
            </svg>
            复制
          </button>
        </div>
        <div ref="editorContainer" class="codemirror-wrapper"></div>
      </div>

      <!-- 输出面板 -->
      <div v-show="activeTab === 'output'" class="editor-panel">
        <div class="editor-actions">
          <button class="btn-action" @click="copyToClipboard(jsonOutput)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
            </svg>
            复制
          </button>
        </div>
        <pre class="output-content">{{ jsonOutput || '暂无输出' }}</pre>
      </div>
    </div>
  </div>
</template>

<style scoped>
.json-editor-plugin {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #ffffff;
}

/* 顶部标题栏 */
.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: #f5f5f7;
  border-bottom: 1px solid #d1d1d6;
}

.header-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.editor-title {
  font-size: 18px;
  font-weight: 600;
  color: #1d1d1f;
  margin: 0;
}

.editor-subtitle {
  font-size: 12px;
  color: #86868b;
}

.btn-new-window {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.04);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 8px;
  color: #1d1d1f;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-new-window:hover {
  background: rgba(0, 122, 255, 0.1);
  border-color: #007AFF;
  color: #007AFF;
}

.btn-close {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.04);
  border: 1px solid rgba(0, 0, 0, 0.08);
  border-radius: 8px;
  color: #1d1d1f;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-close:hover {
  background: rgba(0, 0, 0, 0.08);
}

/* 工具栏 */
.toolbar {
  display: flex;
  justify-content: space-between;
  padding: 12px 24px;
  background: #fafafa;
  border-bottom: 1px solid #e5e5e7;
}

.toolbar-group {
  display: flex;
  gap: 8px;
}

.btn-tool {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: #ffffff;
  border: 1px solid #d1d1d6;
  border-radius: 6px;
  color: #1d1d1f;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-tool:hover {
  background: #f5f5f7;
  border-color: #007AFF;
}

/* 筛选栏 */
.filter-bar {
  padding: 16px 24px;
  background: #fafafa;
  border-bottom: 1px solid #e5e5e7;
}

.filter-input-group {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.filter-icon {
  color: #86868b;
  flex-shrink: 0;
}

.filter-input {
  flex: 1;
  padding: 10px 14px;
  background: #ffffff;
  border: 1px solid #d1d1d6;
  border-radius: 8px;
  color: #1d1d1f;
  font-size: 13px;
  outline: none;
  transition: all 0.2s ease;
}

.filter-input:focus {
  border-color: #007AFF;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.15);
}

.btn-filter {
  padding: 10px 20px;
  background: #007AFF;
  border: none;
  border-radius: 8px;
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-filter:hover {
  background: #0051D5;
}

.filter-hint {
  font-size: 12px;
  color: #86868b;
}

.filter-hint code {
  padding: 2px 6px;
  background: #f5f5f7;
  border: 1px solid #e5e5e7;
  border-radius: 4px;
  font-family: "SF Mono", Monaco, monospace;
  font-size: 11px;
  color: #007AFF;
}

/* 消息提示 */
.message {
  margin: 12px 24px;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 13px;
}

.error-message {
  background: #fff5f5;
  border: 1px solid #ffcccc;
  color: #d32f2f;
}

.success-message {
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
  color: #16a34a;
}

/* 编辑器容器 */
.editor-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #ffffff;
}

.tabs {
  display: flex;
  gap: 4px;
  padding: 12px 24px 0;
  background: #fafafa;
  border-bottom: 1px solid #e5e5e7;
}

.tab {
  position: relative;
  padding: 10px 20px;
  background: transparent;
  border: none;
  border-radius: 8px 8px 0 0;
  color: #86868b;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab:hover {
  background: rgba(0, 0, 0, 0.04);
  color: #1d1d1f;
}

.tab.active {
  background: #ffffff;
  color: #1d1d1f;
  border-bottom: 2px solid #007AFF;
}

.tab-badge {
  margin-left: 6px;
  color: #007AFF;
  font-size: 18px;
}

.editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  overflow: hidden;
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  padding: 12px 24px;
  background: #fafafa;
  border-bottom: 1px solid #e5e5e7;
}

.btn-action {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #ffffff;
  border: 1px solid #d1d1d6;
  border-radius: 6px;
  color: #1d1d1f;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-action:hover {
  background: #f5f5f7;
  border-color: #007AFF;
}

.codemirror-wrapper {
  flex: 1;
  overflow: auto;
  background: #ffffff;
}

.codemirror-wrapper :deep(.cm-editor) {
  height: 100%;
  font-size: 13px;
  font-family: "SF Mono", Monaco, monospace;
  background: #ffffff;
}

.codemirror-wrapper :deep(.cm-scroller) {
  overflow: auto;
}

.codemirror-wrapper :deep(.cm-gutters) {
  background: #fafafa;
  border-right: 1px solid #e5e5e7;
}

.codemirror-wrapper :deep(.cm-activeLineGutter) {
  background: #f5f5f7;
}

.codemirror-wrapper :deep(.cm-activeLine) {
  background: #f5f5f7;
}

.output-content {
  flex: 1;
  margin: 0;
  padding: 24px;
  background: #ffffff;
  color: #1d1d1f;
  font-size: 13px;
  font-family: "SF Mono", Monaco, monospace;
  line-height: 1.6;
  overflow: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}

/* 滚动条 */
.codemirror-wrapper::-webkit-scrollbar,
.output-content::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

.codemirror-wrapper::-webkit-scrollbar-track,
.output-content::-webkit-scrollbar-track {
  background: #fafafa;
}

.codemirror-wrapper::-webkit-scrollbar-thumb,
.output-content::-webkit-scrollbar-thumb {
  background: #d1d1d6;
  border-radius: 4px;
}

.codemirror-wrapper::-webkit-scrollbar-thumb:hover,
.output-content::-webkit-scrollbar-thumb:hover {
  background: #86868b;
}
</style>
