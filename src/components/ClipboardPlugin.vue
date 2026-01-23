<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  getClipboardHistory,
  addClipboardItem,
  deleteClipboardItem,
  clearClipboardHistory,
  toggleClipboardFavorite,
  type ClipboardItem,
} from "../db";

const emit = defineEmits<{
  close: [];
}>();

const clipboardHistory = ref<ClipboardItem[]>([]);
const searchQuery = ref("");
const selectedIndex = ref(0);
const showFavoritesOnly = ref(false);

// 过滤后的历史记录
const filteredHistory = computed(() => {
  let items = clipboardHistory.value;
  
  // 收藏过滤
  if (showFavoritesOnly.value) {
    items = items.filter(item => item.favorite);
  }
  
  // 搜索过滤
  if (searchQuery.value) {
    items = items.filter(item =>
      item.content.toLowerCase().includes(searchQuery.value.toLowerCase())
    );
  }
  
  return items;
});

// 加载剪贴板历史
async function loadHistory() {
  try {
    clipboardHistory.value = await getClipboardHistory();
  } catch (e) {
    console.error("Failed to load clipboard history:", e);
  }
}

// 复制到剪贴板
async function copyToClipboard(item: ClipboardItem) {
  try {
    await invoke("write_clipboard_text", { text: item.content });
    console.log("Copied to clipboard:", item.content);
    emit("close");
  } catch (e) {
    console.error("Failed to copy to clipboard:", e);
  }
}

// 删除项
async function deleteItem(id: string) {
  try {
    await deleteClipboardItem(id);
    await loadHistory();
  } catch (e) {
    console.error("Failed to delete item:", e);
  }
}

// 清空历史
async function clearHistory() {
  if (confirm("确定要清空所有剪贴板历史吗？")) {
    try {
      await clearClipboardHistory();
      await loadHistory();
    } catch (e) {
      console.error("Failed to clear history:", e);
    }
  }
}

// 切换收藏
async function toggleFavorite(id: string) {
  try {
    await toggleClipboardFavorite(id);
    await loadHistory();
  } catch (e) {
    console.error("Failed to toggle favorite:", e);
  }
}

// 监听剪贴板变化
let clipboardInterval: number | null = null;
let lastClipboardContent = "";

async function checkClipboard() {
  try {
    const content = await invoke<string>("read_clipboard_text");
    if (content && content !== lastClipboardContent && content.trim()) {
      lastClipboardContent = content;
      await addClipboardItem({
        content,
        contentType: "text",
      });
      await loadHistory();
      console.log("Added clipboard item:", content.substring(0, 50));
    }
  } catch (e) {
    // 忽略读取错误
  }
}

async function startClipboardMonitor() {
  // 立即检查一次
  await checkClipboard();
  
  // 然后每秒检查一次
  clipboardInterval = setInterval(checkClipboard, 1000) as unknown as number;
}

function stopClipboardMonitor() {
  if (clipboardInterval) {
    clearInterval(clipboardInterval);
    clipboardInterval = null;
  }
}

// 格式化文件大小
function formatFileSize(bytes?: number): string {
  if (!bytes) return "";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

// 格式化时间
function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  
  if (diff < 60000) return "刚刚";
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`;
  if (diff < 604800000) return `${Math.floor(diff / 86400000)} 天前`;
  
  return date.toLocaleDateString();
}

// 键盘导航
function handleKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowDown") {
    e.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, filteredHistory.value.length - 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
  } else if (e.key === "Enter" && filteredHistory.value.length > 0) {
    copyToClipboard(filteredHistory.value[selectedIndex.value]);
  } else if (e.key === "Escape") {
    emit("close");
  }
}

onMounted(async () => {
  await loadHistory();
  startClipboardMonitor();
  window.addEventListener("keydown", handleKeydown);
  
  // 初始化最后的剪贴板内容
  try {
    lastClipboardContent = await invoke<string>("read_clipboard_text");
  } catch (e) {
    // 忽略
  }
});

onUnmounted(() => {
  stopClipboardMonitor();
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <div class="clipboard-plugin">
    <!-- 顶部工具栏 -->
    <div class="plugin-header" data-tauri-drag-region>
      <div class="header-left">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
          <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
        </svg>
        <span class="plugin-title">剪贴板历史</span>
        <span class="item-count">{{ filteredHistory.length }} 项</span>
      </div>
      <div class="header-actions">
        <button
          class="action-btn"
          :class="{ active: showFavoritesOnly }"
          @click="showFavoritesOnly = !showFavoritesOnly"
          title="只看收藏"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
          </svg>
        </button>
        <button class="action-btn" @click="clearHistory" title="清空历史">
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

    <!-- 搜索栏 -->
    <div class="search-section">
      <svg class="search-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"/>
        <path d="m21 21-4.35-4.35"/>
      </svg>
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        placeholder="搜索剪贴板内容..."
      />
    </div>

    <!-- 历史记录列表 -->
    <div class="history-list">
      <div
        v-for="(item, index) in filteredHistory"
        :key="item.id"
        class="history-item"
        :class="{ selected: index === selectedIndex }"
        @click="copyToClipboard(item)"
      >
        <div class="item-content">
          <div v-if="item.contentType === 'image'" class="content-image">
            <img v-if="item.filePath" :src="item.filePath" alt="图片" />
            <div class="image-info">
              <span class="type-badge">🖼️ 图片</span>
              <span v-if="item.fileSize" class="file-size">{{ formatFileSize(item.fileSize) }}</span>
            </div>
          </div>
          <div v-else-if="item.contentType === 'file'" class="content-file">
            <div class="file-icon">📄</div>
            <div class="file-info">
              <div class="file-name">{{ item.content }}</div>
              <span v-if="item.fileSize" class="file-size">{{ formatFileSize(item.fileSize) }}</span>
            </div>
          </div>
          <div v-else class="content-text">{{ item.content }}</div>
          <div class="item-meta">
            <span class="item-time">{{ formatTime(item.timestamp) }}</span>
            <span class="item-type">{{ item.contentType }}</span>
          </div>
        </div>
        <div class="item-actions">
          <button
            class="item-action-btn"
            :class="{ active: item.favorite }"
            @click.stop="toggleFavorite(item.id)"
            title="收藏"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" :fill="item.favorite ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
            </svg>
          </button>
          <button
            class="item-action-btn delete-btn"
            @click.stop="deleteItem(item.id)"
            title="删除"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18"/>
              <line x1="6" y1="6" x2="18" y2="18"/>
            </svg>
          </button>
        </div>
      </div>

      <div v-if="filteredHistory.length === 0" class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
          <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
        </svg>
        <p>暂无剪贴板历史</p>
        <span>复制内容后会自动记录</span>
      </div>
    </div>

    <!-- 底部提示 -->
    <div class="plugin-footer">
      <div class="footer-hint">
        <kbd>↑</kbd> <kbd>↓</kbd> 选择 · <kbd>Enter</kbd> 复制 · <kbd>Esc</kbd> 关闭
      </div>
    </div>
  </div>
</template>

<style scoped>
.clipboard-plugin {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #2b2b2b;
}

/* 顶部工具栏 */
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

.item-count {
  font-size: 12px;
  color: #888;
  padding: 2px 8px;
  background: #2b2b2b;
  border-radius: 10px;
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
  -webkit-app-region: no-drag;
}

.action-btn:hover {
  background: #4a4a4a;
  color: #e0e0e0;
  border-color: #5a5a5a;
}

.action-btn.active {
  background: #5a9fd4;
  color: #fff;
  border-color: #5a9fd4;
}

.action-btn.close-btn:hover {
  background: #e74c3c;
  border-color: #e74c3c;
  color: #fff;
}

/* 搜索栏 */
.search-section {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  background: #2b2b2b;
  border-bottom: 1px solid #3a3a3a;
}

.search-icon {
  color: #888;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  padding: 8px 12px;
  font-size: 13px;
  color: #e0e0e0;
  outline: none;
  transition: all 0.2s ease;
}

.search-input:focus {
  border-color: #5a9fd4;
  box-shadow: 0 0 0 2px rgba(90, 159, 212, 0.2);
}

.search-input::placeholder {
  color: #666;
}

/* 历史记录列表 */
.history-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: #323232;
  border: 1px solid #3a3a3a;
  border-radius: 8px;
  margin-bottom: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.history-item:hover,
.history-item.selected {
  background: #3a3a3a;
  border-color: #4a4a4a;
}

.history-item.selected {
  border-color: #5a9fd4;
  box-shadow: 0 0 0 1px #5a9fd4 inset;
}

.item-content {
  flex: 1;
  min-width: 0;
}

.content-text {
  font-size: 13px;
  color: #e0e0e0;
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  word-break: break-all;
}

.content-image {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.content-image img {
  max-width: 200px;
  max-height: 120px;
  border-radius: 6px;
  object-fit: cover;
  background: #1a1a1a;
}

.image-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.type-badge {
  font-size: 11px;
  color: #5a9fd4;
  padding: 2px 6px;
  background: rgba(90, 159, 212, 0.1);
  border-radius: 4px;
}

.file-size {
  font-size: 11px;
  color: #888;
}

.content-file {
  display: flex;
  align-items: center;
  gap: 12px;
}

.file-icon {
  font-size: 32px;
  flex-shrink: 0;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-size: 13px;
  color: #e0e0e0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 6px;
}

.item-time,
.item-type {
  font-size: 11px;
  color: #666;
}

.item-actions {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.item-action-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 4px;
  color: #666;
  cursor: pointer;
  transition: all 0.2s ease;
}

.item-action-btn:hover {
  background: #4a4a4a;
  color: #e0e0e0;
  border-color: #5a5a5a;
}

.item-action-btn.active {
  color: #f5a623;
}

.item-action-btn.delete-btn:hover {
  background: #e74c3c;
  border-color: #e74c3c;
  color: #fff;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: #666;
  text-align: center;
}

.empty-state svg {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 14px;
  margin: 0 0 8px 0;
}

.empty-state span {
  font-size: 12px;
  opacity: 0.7;
}

/* 底部提示 */
.plugin-footer {
  padding: 12px 20px;
  background: #323232;
  border-top: 1px solid #3a3a3a;
}

.footer-hint {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 12px;
  color: #888;
}

.footer-hint kbd {
  padding: 2px 6px;
  background: #2b2b2b;
  border: 1px solid #4a4a4a;
  border-radius: 4px;
  font-family: monospace;
  font-size: 11px;
  color: #e0e0e0;
}

/* 滚动条 */
.history-list::-webkit-scrollbar {
  width: 6px;
}

.history-list::-webkit-scrollbar-track {
  background: transparent;
}

.history-list::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 3px;
}

.history-list::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}
</style>
