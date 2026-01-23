<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { JSONEditor, Mode, type Content, type OnChangeStatus } from 'vanilla-jsoneditor';

const emit = defineEmits<{
  close: [];
}>();

const editorContainer = ref<HTMLDivElement | null>(null);
const searchPath = ref("");
const searchResult = ref("");
const errorMessage = ref("");

let editor: JSONEditor | null = null;

// 初始化编辑器
onMounted(() => {
  if (editorContainer.value) {
    editor = new JSONEditor({
      target: editorContainer.value,
      props: {
        mode: Mode.tree,
        mainMenuBar: true,
        navigationBar: true,
        statusBar: true,
        readOnly: false,
        content: {
          text: JSON.stringify({
            "示例": "在这里粘贴或编辑 JSON 数据",
            "功能": {
              "格式化": "自动格式化 JSON",
              "验证": "实时验证 JSON 语法",
              "搜索": "支持路径搜索",
              "树形视图": "可视化编辑"
            },
            "快捷键": {
              "Ctrl+F": "搜索",
              "Ctrl+H": "替换",
              "Ctrl+Z": "撤销",
              "Ctrl+Y": "重做"
            }
          }, null, 2)
        },
        onChange: (content: Content, previousContent: Content, status: OnChangeStatus) => {
          errorMessage.value = "";
          if (status.contentErrors) {
            errorMessage.value = status.contentErrors.validationErrors?.[0]?.message || "JSON 格式错误";
          }
        }
      }
    });
  }
});

onUnmounted(() => {
  if (editor) {
    editor.destroy();
  }
});

// 按路径搜索
function searchByPath() {
  if (!editor || !searchPath.value) {
    searchResult.value = "";
    return;
  }

  try {
    const content = editor.get();
    if (!content.json) {
      searchResult.value = "请先输入有效的 JSON";
      return;
    }

    const path = searchPath.value.trim();
    const keys = path.split('.').filter(k => k);
    
    let result: any = content.json;
    for (const key of keys) {
      // 支持数组索引
      if (key.includes('[') && key.includes(']')) {
        const arrayKey = key.substring(0, key.indexOf('['));
        const index = parseInt(key.substring(key.indexOf('[') + 1, key.indexOf(']')));
        
        if (arrayKey) {
          result = result[arrayKey];
        }
        
        if (Array.isArray(result) && !isNaN(index)) {
          result = result[index];
        } else {
          searchResult.value = `路径 "${path}" 不是有效的数组索引`;
          return;
        }
      } else {
        if (result && typeof result === 'object' && key in result) {
          result = result[key];
        } else {
          searchResult.value = `路径 "${path}" 不存在`;
          return;
        }
      }
    }

    searchResult.value = JSON.stringify(result, null, 2);
  } catch (e: any) {
    searchResult.value = `搜索失败: ${e.message}`;
  }
}

// 复制结果
async function copyResult() {
  if (!searchResult.value) return;
  
  try {
    await navigator.clipboard.writeText(searchResult.value);
    alert("已复制到剪贴板");
  } catch (e) {
    console.error("Failed to copy:", e);
  }
}

// 格式化 JSON
function formatJson() {
  if (!editor) return;
  
  try {
    const content = editor.get();
    if (content.json) {
      editor.set({
        json: content.json
      });
    }
  } catch (e: any) {
    errorMessage.value = `格式化失败: ${e.message}`;
  }
}

// 压缩 JSON
function minifyJson() {
  if (!editor) return;
  
  try {
    const content = editor.get();
    if (content.json) {
      editor.set({
        text: JSON.stringify(content.json)
      });
    }
  } catch (e: any) {
    errorMessage.value = `压缩失败: ${e.message}`;
  }
}

// 清空编辑器
function clearEditor() {
  if (!editor) return;
  
  if (confirm("确定要清空编辑器吗？")) {
    editor.set({
      text: ""
    });
    searchPath.value = "";
    searchResult.value = "";
    errorMessage.value = "";
  }
}

// 从剪贴板粘贴
async function pasteFromClipboard() {
  if (!editor) return;
  
  try {
    const text = await navigator.clipboard.readText();
    const json = JSON.parse(text);
    editor.set({
      json: json
    });
  } catch (e: any) {
    errorMessage.value = `粘贴失败: ${e.message}`;
  }
}

// 复制到剪贴板
async function copyToClipboard() {
  if (!editor) return;
  
  try {
    const content = editor.get();
    const text = content.text || JSON.stringify(content.json, null, 2);
    await navigator.clipboard.writeText(text);
    alert("已复制到剪贴板");
  } catch (e: any) {
    errorMessage.value = `复制失败: ${e.message}`;
  }
}
</script>

<template>
  <div class="json-editor-plugin">
    <!-- 顶部工具栏 -->
    <div class="plugin-header" data-tauri-drag-region>
      <div class="header-left">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M4 7h16M4 12h16M4 17h16"/>
        </svg>
        <span class="plugin-title">JSON 编辑器</span>
      </div>
      <div class="header-actions">
        <button class="action-btn" @click="pasteFromClipboard" title="从剪贴板粘贴">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
            <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
          </svg>
        </button>
        <button class="action-btn" @click="copyToClipboard" title="复制到剪贴板">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
          </svg>
        </button>
        <button class="action-btn" @click="formatJson" title="格式化">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="16 18 22 12 16 6"/>
            <polyline points="8 6 2 12 8 18"/>
          </svg>
        </button>
        <button class="action-btn" @click="minifyJson" title="压缩">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
        </button>
        <button class="action-btn" @click="clearEditor" title="清空">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
        </button>
        <button class="action-btn close-btn" @click="emit('close')" title="关闭">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- 路径搜索 -->
    <div class="search-panel">
      <div class="search-input-group">
        <input
          v-model="searchPath"
          type="text"
          class="path-input"
          placeholder="输入路径搜索，例如: user.name 或 items[0].title"
          @keyup.enter="searchByPath"
        />
        <button class="search-btn" @click="searchByPath">搜索</button>
      </div>
      
      <div v-if="searchResult" class="search-result">
        <div class="result-header">
          <span class="result-label">搜索结果:</span>
          <button class="copy-btn" @click="copyResult">复制</button>
        </div>
        <pre class="result-content">{{ searchResult }}</pre>
      </div>
    </div>

    <!-- 错误提示 -->
    <div v-if="errorMessage" class="error-banner">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"/>
        <line x1="12" y1="8" x2="12" y2="12"/>
        <line x1="12" y1="16" x2="12.01" y2="16"/>
      </svg>
      <span>{{ errorMessage }}</span>
    </div>

    <!-- JSON 编辑器 -->
    <div ref="editorContainer" class="editor-container"></div>
  </div>
</template>

<style scoped>
.json-editor-plugin {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #2b2b2b;
}

.plugin-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: #3a3a3a;
  border-bottom: 1px solid #4a4a4a;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #e0e0e0;
}

.plugin-title {
  font-size: 14px;
  font-weight: 500;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.action-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  color: #888;
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: #4a4a4a;
  color: #e0e0e0;
  border-color: #5a5a5a;
}

.action-btn.close-btn:hover {
  background: #e74c3c;
  border-color: #e74c3c;
  color: #fff;
}

.search-panel {
  padding: 12px 20px;
  background: #323232;
  border-bottom: 1px solid #3a3a3a;
}

.search-input-group {
  display: flex;
  gap: 8px;
}

.path-input {
  flex: 1;
  background: #2b2b2b;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  padding: 8px 12px;
  font-size: 13px;
  color: #e0e0e0;
  font-family: monospace;
  outline: none;
  transition: all 0.2s ease;
}

.path-input:focus {
  border-color: #5a9fd4;
  box-shadow: 0 0 0 2px rgba(90, 159, 212, 0.2);
}

.search-btn {
  padding: 8px 16px;
  background: #5a9fd4;
  border: none;
  border-radius: 6px;
  color: #fff;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.search-btn:hover {
  background: #4a8fc4;
}

.search-result {
  margin-top: 12px;
  background: #2b2b2b;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  overflow: hidden;
}

.result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #323232;
  border-bottom: 1px solid #3a3a3a;
}

.result-label {
  font-size: 12px;
  color: #888;
  font-weight: 500;
}

.copy-btn {
  padding: 4px 12px;
  background: transparent;
  border: 1px solid #4a4a4a;
  border-radius: 4px;
  color: #888;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.copy-btn:hover {
  background: #4a4a4a;
  color: #e0e0e0;
}

.result-content {
  padding: 12px;
  margin: 0;
  font-size: 12px;
  font-family: monospace;
  color: #e0e0e0;
  max-height: 150px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}

.error-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  background: rgba(231, 76, 60, 0.1);
  border-bottom: 1px solid #e74c3c;
  color: #e74c3c;
  font-size: 13px;
}

.editor-container {
  flex: 1;
  overflow: hidden;
}

/* 覆盖 vanilla-jsoneditor 的样式 */
.editor-container :deep(.jse-theme-dark) {
  --jse-theme-color: #5a9fd4;
  --jse-theme-color-highlight: #4a8fc4;
  --jse-background-color: #2b2b2b;
  --jse-panel-background: #323232;
  --jse-panel-border: #4a4a4a;
  --jse-text-color: #e0e0e0;
  --jse-text-color-inverse: #2b2b2b;
}
</style>
