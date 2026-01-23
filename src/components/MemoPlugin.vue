<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  getMemos,
  addMemo,
  updateMemo,
  deleteMemo,
  toggleMemoPinned,
  type MemoItem,
} from "../db";

const emit = defineEmits<{
  close: [];
}>();

const memos = ref<MemoItem[]>([]);
const searchQuery = ref("");
const showEditor = ref(false);
const editingMemo = ref<MemoItem | null>(null);

// 编辑表单
const memoForm = ref({
  title: "",
  content: "",
  tags: [] as string[],
});

const tagInput = ref("");

// 自动同步相关
const yxbjConfigured = ref(false);
const autoSyncEnabled = ref(true); // 自动同步开关
const syncStatus = ref<"idle" | "syncing" | "success" | "error">("idle");
const lastSyncTime = ref<number>(0);
let syncTimer: number | null = null;
let idleTimer: number | null = null;

// 过滤后的备忘录
const filteredMemos = computed(() => {
  let items = memos.value;
  
  if (searchQuery.value) {
    items = items.filter(memo =>
      memo.title.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      memo.content.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      memo.tags.some(tag => tag.toLowerCase().includes(searchQuery.value.toLowerCase()))
    );
  }
  
  return items;
});

// 检查印象笔记是否配置
async function checkYxbjConfig() {
  try {
    const configStr = await invoke<string>("load_mcp_config");
    const config = JSON.parse(configStr);
    yxbjConfigured.value = !!config.mcpServers?.["yxbj-mcp"];
    console.log("YXBJ configured:", yxbjConfigured.value);
  } catch (e) {
    console.error("Failed to check YXBJ config:", e);
    yxbjConfigured.value = false;
  }
}

// 加载备忘录
async function loadMemos() {
  try {
    memos.value = await getMemos();
  } catch (e) {
    console.error("Failed to load memos:", e);
  }
}

// 新建备忘录
function createNewMemo() {
  editingMemo.value = null;
  memoForm.value = {
    title: "",
    content: "",
    tags: [],
  };
  showEditor.value = true;
}

// 编辑备忘录
function editMemo(memo: MemoItem) {
  editingMemo.value = memo;
  memoForm.value = {
    title: memo.title,
    content: memo.content,
    tags: [...memo.tags],
  };
  showEditor.value = true;
}

// 保存备忘录（始终保存到本地）
async function saveMemo() {
  if (!memoForm.value.title.trim()) {
    alert("请输入标题");
    return;
  }

  try {
    if (editingMemo.value) {
      // 更新本地
      await updateMemo(editingMemo.value.id, {
        title: memoForm.value.title,
        content: memoForm.value.content,
        tags: memoForm.value.tags,
      });
    } else {
      // 新建到本地
      await addMemo({
        title: memoForm.value.title,
        content: memoForm.value.content,
        tags: memoForm.value.tags,
      });
    }
    
    showEditor.value = false;
    await loadMemos();
    
    // 触发自动同步
    if (yxbjConfigured.value && autoSyncEnabled.value) {
      scheduleSync();
    }
  } catch (e) {
    console.error("Failed to save memo:", e);
    alert("保存失败");
  }
}

// 同步到印象笔记
async function syncToYxbj() {
  if (!yxbjConfigured.value || !autoSyncEnabled.value) {
    return;
  }

  syncStatus.value = "syncing";
  
  try {
    // 获取所有本地笔记
    const allMemos = await getMemos();
    
    // 同步每条笔记（这里简化处理，实际应该记录哪些已同步）
    for (const memo of allMemos) {
      try {
        await invoke<string>("save_memo_to_yxbj", {
          title: memo.title,
          content: memo.content,
          tags: memo.tags,
        });
      } catch (e) {
        console.error(`Failed to sync memo ${memo.id}:`, e);
      }
    }
    
    syncStatus.value = "success";
    lastSyncTime.value = Date.now();
    
    // 3秒后重置状态
    setTimeout(() => {
      if (syncStatus.value === "success") {
        syncStatus.value = "idle";
      }
    }, 3000);
  } catch (e) {
    console.error("Failed to sync to YXBJ:", e);
    syncStatus.value = "error";
    
    // 5秒后重置状态
    setTimeout(() => {
      if (syncStatus.value === "error") {
        syncStatus.value = "idle";
      }
    }, 5000);
  }
}

// 调度同步（用户停止操作后 3 秒执行）
function scheduleSync() {
  // 清除之前的定时器
  if (syncTimer) {
    clearTimeout(syncTimer);
  }
  
  // 3秒后执行同步
  syncTimer = window.setTimeout(() => {
    syncToYxbj();
  }, 3000);
}

// 监听编辑器内容变化，重置空闲计时器
watch([() => memoForm.value.title, () => memoForm.value.content], () => {
  if (showEditor.value) {
    // 用户正在编辑，重置空闲计时器
    if (idleTimer) {
      clearTimeout(idleTimer);
    }
    
    // 5秒无操作后触发同步
    idleTimer = window.setTimeout(() => {
      if (yxbjConfigured.value && autoSyncEnabled.value) {
        scheduleSync();
      }
    }, 5000);
  }
});

// 删除备忘录
async function deleteMemoItem(id: string) {
  if (confirm("确定要删除这条备忘录吗？")) {
    try {
      await deleteMemo(id);
      await loadMemos();
    } catch (e) {
      console.error("Failed to delete memo:", e);
    }
  }
}

// 切换置顶
async function togglePin(id: string) {
  try {
    await toggleMemoPinned(id);
    await loadMemos();
  } catch (e) {
    console.error("Failed to toggle pin:", e);
  }
}

// 添加标签
function addTag() {
  const tag = tagInput.value.trim();
  if (tag && !memoForm.value.tags.includes(tag)) {
    memoForm.value.tags.push(tag);
    tagInput.value = "";
  }
}

// 删除标签
function removeTag(tag: string) {
  memoForm.value.tags = memoForm.value.tags.filter(t => t !== tag);
}

// 格式化时间
function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  return date.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

// 格式化同步时间
function formatSyncTime(): string {
  if (!lastSyncTime.value) return "从未同步";
  
  const now = Date.now();
  const diff = now - lastSyncTime.value;
  
  if (diff < 60000) return "刚刚";
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`;
  return `${Math.floor(diff / 86400000)} 天前`;
}

onMounted(async () => {
  await loadMemos();
  await checkYxbjConfig();
});

onUnmounted(() => {
  if (syncTimer) clearTimeout(syncTimer);
  if (idleTimer) clearTimeout(idleTimer);
});
</script>

<template>
  <div class="memo-plugin">
    <!-- 列表视图 -->
    <div v-if="!showEditor" class="memo-list-view">
      <!-- 顶部工具栏 -->
      <div class="plugin-header" data-tauri-drag-region>
        <div class="header-left">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
            <line x1="16" y1="13" x2="8" y2="13"/>
            <line x1="16" y1="17" x2="8" y2="17"/>
            <polyline points="10 9 9 9 8 9"/>
          </svg>
          <span class="plugin-title">备忘快贴</span>
          <span class="item-count">{{ filteredMemos.length }} 条</span>
          
          <!-- 同步状态指示器 -->
          <div v-if="yxbjConfigured && autoSyncEnabled" class="sync-indicator" :class="syncStatus">
            <svg v-if="syncStatus === 'syncing'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spin">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
            <svg v-else-if="syncStatus === 'success'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12"/>
            </svg>
            <svg v-else-if="syncStatus === 'error'" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <line x1="12" y1="8" x2="12" y2="12"/>
              <line x1="12" y1="16" x2="12.01" y2="16"/>
            </svg>
            <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2L2 7l10 5 10-5-10-5z"/>
              <path d="M2 17l10 5 10-5"/>
              <path d="M2 12l10 5 10-5"/>
            </svg>
            <span class="sync-text">
              {{ syncStatus === 'syncing' ? '同步中...' : 
                 syncStatus === 'success' ? '已同步' : 
                 syncStatus === 'error' ? '同步失败' : 
                 formatSyncTime() }}
            </span>
          </div>
        </div>
        <div class="header-actions">
          <button 
            v-if="yxbjConfigured" 
            class="action-btn sync-btn" 
            @click="syncToYxbj" 
            :disabled="syncStatus === 'syncing'"
            title="立即同步到印象笔记"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :class="{ spin: syncStatus === 'syncing' }">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
          </button>
          <button class="action-btn primary" @click="createNewMemo" title="新建备忘录">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19"/>
              <line x1="5" y1="12" x2="19" y2="12"/>
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
          placeholder="搜索备忘录..."
        />
      </div>

      <!-- 备忘录列表 -->
      <div class="memos-list">
        <div
          v-for="memo in filteredMemos"
          :key="memo.id"
          class="memo-card"
          :class="{ pinned: memo.pinned }"
          @click="editMemo(memo)"
        >
          <div class="memo-header">
            <h3 class="memo-title">
              <svg v-if="memo.pinned" width="14" height="14" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" stroke-width="2">
                <path d="M12 2L2 7l10 5 10-5-10-5z"/>
              </svg>
              {{ memo.title }}
            </h3>
            <div class="memo-actions">
              <button
                class="memo-action-btn"
                :class="{ active: memo.pinned }"
                @click.stop="togglePin(memo.id)"
                title="置顶"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M12 2L2 7l10 5 10-5-10-5z"/>
                </svg>
              </button>
              <button
                class="memo-action-btn delete-btn"
                @click.stop="deleteMemoItem(memo.id)"
                title="删除"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"/>
                  <line x1="6" y1="6" x2="18" y2="18"/>
                </svg>
              </button>
            </div>
          </div>
          
          <p class="memo-content">{{ memo.content }}</p>
          
          <div class="memo-footer">
            <div class="memo-tags">
              <span v-for="tag in memo.tags" :key="tag" class="tag">{{ tag }}</span>
            </div>
            <span class="memo-time">{{ formatTime(memo.updatedAt) }}</span>
          </div>
        </div>

        <div v-if="filteredMemos.length === 0" class="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
          </svg>
          <p>暂无备忘录</p>
          <button class="create-btn" @click="createNewMemo">创建第一条备忘录</button>
        </div>
      </div>
    </div>

    <!-- 编辑器视图 -->
    <div v-else class="memo-editor-view">
      <!-- 编辑器工具栏 -->
      <div class="editor-header" data-tauri-drag-region>
        <button class="back-btn" @click="showEditor = false">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="15 18 9 12 15 6"/>
          </svg>
          返回
        </button>
        <div class="editor-actions">
          <button class="save-btn" @click="saveMemo">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
              <polyline points="17 21 17 13 7 13 7 21"/>
              <polyline points="7 3 7 8 15 8"/>
            </svg>
            保存
          </button>
        </div>
      </div>

      <!-- 编辑器内容 -->
      <div class="editor-content">
        <input
          v-model="memoForm.title"
          type="text"
          class="title-input"
          placeholder="标题"
        />
        
        <textarea
          v-model="memoForm.content"
          class="content-textarea"
          placeholder="内容..."
        />

        <div class="tags-section">
          <div class="tags-list">
            <span v-for="tag in memoForm.tags" :key="tag" class="tag">
              {{ tag }}
              <button class="tag-remove" @click="removeTag(tag)">×</button>
            </span>
          </div>
          <div class="tag-input-wrapper">
            <input
              v-model="tagInput"
              type="text"
              class="tag-input"
              placeholder="添加标签..."
              @keydown.enter="addTag"
            />
            <button class="add-tag-btn" @click="addTag">添加</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.memo-plugin {
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

/* 同步状态指示器 */
.sync-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: #2b2b2b;
  border-radius: 10px;
  font-size: 11px;
  color: #888;
}

.sync-indicator.syncing {
  color: #5a9fd4;
}

.sync-indicator.success {
  color: #27ae60;
}

.sync-indicator.error {
  color: #e74c3c;
}

.sync-indicator svg {
  flex-shrink: 0;
}

.sync-text {
  white-space: nowrap;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.spin {
  animation: spin 1s linear infinite;
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

.action-btn.primary {
  background: #5a9fd4;
  color: #fff;
  border-color: #5a9fd4;
}

.action-btn.primary:hover {
  background: #6aafed;
  border-color: #6aafed;
}

.action-btn.sync-btn {
  background: transparent;
  color: #888;
  border-color: #4a4a4a;
}

.action-btn.sync-btn:hover:not(:disabled) {
  background: #4a4a4a;
  color: #5a9fd4;
  border-color: #5a5a5a;
}

.action-btn.sync-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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

/* 备忘录列表 */
.memos-list {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.memo-card {
  padding: 16px;
  background: #323232;
  border: 1px solid #3a3a3a;
  border-radius: 8px;
  margin-bottom: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.memo-card:hover {
  background: #3a3a3a;
  border-color: #4a4a4a;
  transform: translateY(-2px);
}

.memo-card.pinned {
  border-color: #5a9fd4;
  box-shadow: 0 0 0 1px #5a9fd4 inset;
}

.memo-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.memo-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 15px;
  font-weight: 500;
  color: #e0e0e0;
  margin: 0;
}

.memo-title svg {
  color: #5a9fd4;
}

.memo-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.memo-action-btn {
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

.memo-action-btn:hover {
  background: #4a4a4a;
  color: #e0e0e0;
  border-color: #5a5a5a;
}

.memo-action-btn.active {
  color: #5a9fd4;
}

.memo-action-btn.delete-btn:hover {
  background: #e74c3c;
  border-color: #e74c3c;
  color: #fff;
}

.memo-content {
  font-size: 13px;
  color: #b0b0b0;
  line-height: 1.6;
  margin: 0 0 12px 0;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}

.memo-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.memo-tags {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.tag {
  padding: 2px 8px;
  background: #2b2b2b;
  border: 1px solid #4a4a4a;
  border-radius: 10px;
  font-size: 11px;
  color: #888;
}

.memo-time {
  font-size: 11px;
  color: #666;
  white-space: nowrap;
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
  margin: 0 0 16px 0;
}

.create-btn {
  padding: 10px 20px;
  background: #5a9fd4;
  border: none;
  border-radius: 6px;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.create-btn:hover {
  background: #6aafed;
}

/* 编辑器 */
.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: #3a3a3a;
  border-bottom: 1px solid #4a4a4a;
}

.back-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: transparent;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  color: #e0e0e0;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.back-btn:hover {
  background: #4a4a4a;
  border-color: #5a5a5a;
}

.save-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: #5a9fd4;
  border: none;
  border-radius: 6px;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.save-btn:hover {
  background: #6aafed;
}

.editor-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 20px;
  overflow-y: auto;
}

.title-input {
  width: 100%;
  padding: 12px 0;
  background: transparent;
  border: none;
  border-bottom: 2px solid #3a3a3a;
  font-size: 20px;
  font-weight: 600;
  color: #e0e0e0;
  outline: none;
  margin-bottom: 20px;
}

.title-input::placeholder {
  color: #666;
}

.content-textarea {
  flex: 1;
  width: 100%;
  padding: 12px;
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 8px;
  font-size: 14px;
  line-height: 1.6;
  color: #e0e0e0;
  outline: none;
  resize: none;
  font-family: inherit;
  margin-bottom: 20px;
}

.content-textarea::placeholder {
  color: #666;
}

.tags-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.tags-list {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.tags-list .tag {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: 12px;
  font-size: 12px;
  color: #e0e0e0;
}

.tag-remove {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 50%;
  color: #888;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tag-remove:hover {
  background: #4a4a4a;
  color: #e0e0e0;
}

.tag-input-wrapper {
  display: flex;
  gap: 8px;
}

.tag-input {
  flex: 1;
  padding: 8px 12px;
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  font-size: 13px;
  color: #e0e0e0;
  outline: none;
}

.tag-input::placeholder {
  color: #666;
}

.add-tag-btn {
  padding: 8px 16px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  color: #e0e0e0;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.add-tag-btn:hover {
  background: #4a4a4a;
  border-color: #5a5a5a;
}

/* 滚动条 */
.memos-list::-webkit-scrollbar,
.editor-content::-webkit-scrollbar {
  width: 6px;
}

.memos-list::-webkit-scrollbar-track,
.editor-content::-webkit-scrollbar-track {
  background: transparent;
}

.memos-list::-webkit-scrollbar-thumb,
.editor-content::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 3px;
}

.memos-list::-webkit-scrollbar-thumb:hover,
.editor-content::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}
</style>
