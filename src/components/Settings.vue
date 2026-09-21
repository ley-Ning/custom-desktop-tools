<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import McpSettings from "./McpSettings.vue";
import AiModelSettings from "./AiModelSettings.vue";

const emit = defineEmits<{
  close: [];
  openMarket: [];
}>();

// 当前选中的菜单项
const selectedMenu = ref("settings");

// 状态持久化的 key
const SETTINGS_MENU_KEY = "settings_selected_menu";

// 快捷键设置
const currentShortcut = ref<string>("Alt+Space");
const editingShortcut = ref<string>("");
const isEditingShortcut = ref(false);
const shortcutError = ref<string>("");
const shortcutSuccess = ref<string>("");

// 数据统计
const clipboardCount = ref(0);
const memoCount = ref(0);
const clipboardSize = ref("0 KB");
const memoSize = ref("0 KB");

// 加载数据统计
async function loadDataStats() {
  try {
    // 加载剪贴板数量
    const clipboardHistory = await invoke<any[]>("get_clipboard_history");
    clipboardCount.value = clipboardHistory.length;
    
    // 计算剪贴板大小（估算）
    const clipboardBytes = JSON.stringify(clipboardHistory).length;
    clipboardSize.value = formatBytes(clipboardBytes);
    
    // 加载备忘录数量
    const memos = await invoke<any[]>("get_memos");
    memoCount.value = memos.length;
    
    // 计算备忘录大小（估算）
    const memoBytes = JSON.stringify(memos).length;
    memoSize.value = formatBytes(memoBytes);
  } catch (e) {
    console.error("Failed to load data stats:", e);
  }
}

// 格式化字节大小
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 KB";
  const k = 1024;
  if (bytes < k) return "1 KB";
  const sizes = ["KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i - 1];
}

// 清空剪贴板历史
async function clearClipboardHistory() {
  if (confirm("确定要清空所有剪贴板历史吗？此操作不可恢复。")) {
    try {
      await invoke("clear_clipboard_history");
      await loadDataStats();
      alert("剪贴板历史已清空");
    } catch (e) {
      console.error("Failed to clear clipboard:", e);
      alert("清空失败：" + e);
    }
  }
}

onMounted(async () => {
  // 恢复上次选中的菜单
  const savedMenu = localStorage.getItem(SETTINGS_MENU_KEY);
  if (savedMenu) {
    selectedMenu.value = savedMenu;
  }

  try {
    const shortcut = await invoke<string | null>("get_shortcut");
    if (shortcut) {
      currentShortcut.value = shortcut;
      editingShortcut.value = shortcut;
    }
  } catch (e) {
    console.error("Failed to get shortcut:", e);
  }

  // 加载数据统计
  await loadDataStats();

  // 当前版本号
  try {
    appVersion.value = await getVersion();
  } catch (e) {
    console.error("Failed to get version:", e);
  }

  // 托盘「检测更新」→ 切到更新面板并立即检查
  unlistenCheckUpdate = await listen<void>("check-update", () => {
    selectMenu("update");
    checkForUpdate();
  });
});

let unlistenCheckUpdate: (() => void) | null = null;

onUnmounted(() => {
  if (unlistenCheckUpdate) unlistenCheckUpdate();
});

function selectMenu(menu: string) {
  selectedMenu.value = menu;
  // 保存选中的菜单
  localStorage.setItem(SETTINGS_MENU_KEY, menu);
}

// ===== 软件更新（tauri-plugin-updater） =====

type UpdateStatus = "idle" | "checking" | "available" | "uptodate" | "downloading" | "ready" | "error";

const appVersion = ref("");
const updateStatus = ref<UpdateStatus>("idle");
const newVersion = ref("");
const updateNotes = ref("");
const downloadProgress = ref(0);
const downloadedBytes = ref(0);
const totalBytes = ref(0);
const updateError = ref("");
let pendingUpdate: Update | null = null;

async function checkForUpdate() {
  if (updateStatus.value === "checking" || updateStatus.value === "downloading") return;
  updateStatus.value = "checking";
  updateError.value = "";
  try {
    const update = await check();
    if (update) {
      pendingUpdate = update;
      newVersion.value = update.version;
      updateNotes.value = update.body || "";
      updateStatus.value = "available";
    } else {
      pendingUpdate = null;
      updateStatus.value = "uptodate";
    }
  } catch (e) {
    updateStatus.value = "error";
    updateError.value = `${e}`;
  }
}

async function downloadAndInstallUpdate() {
  if (!pendingUpdate) return;
  updateStatus.value = "downloading";
  downloadProgress.value = 0;
  downloadedBytes.value = 0;
  let total = 0;
  let downloaded = 0;
  try {
    await pendingUpdate.downloadAndInstall((event) => {
      switch (event.event) {
        case "Started":
          total = event.data.contentLength ?? 0;
          totalBytes.value = total;
          break;
        case "Progress":
          downloaded += event.data.chunkLength;
          downloadedBytes.value = downloaded;
          downloadProgress.value = total > 0 ? Math.round((downloaded / total) * 100) : 0;
          break;
        case "Finished":
          downloadProgress.value = 100;
          break;
      }
    });
    updateStatus.value = "ready";
  } catch (e) {
    updateStatus.value = "error";
    updateError.value = `${e}`;
  }
}

async function restartApp() {
  try {
    await relaunch();
  } catch (e) {
    console.error("重启失败:", e);
  }
}

function startEditingShortcut() {
  isEditingShortcut.value = true;
  editingShortcut.value = currentShortcut.value;
  shortcutError.value = "";
  shortcutSuccess.value = "";
}

function cancelEditingShortcut() {
  isEditingShortcut.value = false;
  editingShortcut.value = currentShortcut.value;
  shortcutError.value = "";
}

async function saveShortcut() {
  shortcutError.value = "";
  shortcutSuccess.value = "";

  try {
    await invoke("set_shortcut", { shortcut: editingShortcut.value });
    currentShortcut.value = editingShortcut.value;
    isEditingShortcut.value = false;
    shortcutSuccess.value = "快捷键已保存";
    setTimeout(() => {
      shortcutSuccess.value = "";
    }, 2000);
  } catch (e) {
    shortcutError.value = String(e).replace("Error: ", "");
  }
}

function handleShortcutInput(e: KeyboardEvent) {
  e.preventDefault();

  const keys: string[] = [];

  if (e.ctrlKey || e.metaKey) {
    keys.push("CmdOrCtrl");
  }
  if (e.altKey) {
    keys.push("Alt");
  }
  if (e.shiftKey) {
    keys.push("Shift");
  }

  if (e.key && !["Control", "Alt", "Shift", "Meta"].includes(e.key)) {
    keys.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);
  }

  if (keys.length >= 2) {
    editingShortcut.value = keys.join("+");
    shortcutError.value = "";
  } else {
    shortcutError.value = "请至少按下一个修饰键（Cmd/Alt/Shift）和一个字符键";
  }
}
</script>

<template>
  <div class="settings-container">
    <!-- 顶部标题栏 -->
    <div class="settings-header" data-tauri-drag-region>
      <div class="header-left">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2L2 7l10 5 10-5-10-5z"/>
          <path d="M2 17l10 5 10-5"/>
          <path d="M2 12l10 5 10-5"/>
        </svg>
        <span class="header-title">管理中心</span>
      </div>
      <button class="close-btn" @click="emit('close')">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
        <span>设置</span>
      </button>
    </div>

    <div class="settings-body">
      <!-- 左侧导航 -->
      <div class="settings-sidebar">
        <!-- 偏好设置 -->
        <div class="sidebar-section">
          <div class="section-title">偏好设置</div>
          <div
            class="menu-item active-highlight"
            :class="{ active: selectedMenu === 'settings' }"
            @click="selectMenu('settings')"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="3"/>
              <path d="M12 1v6m0 6v6m9-9h-6m-6 0H3m15.364 6.364l-4.243-4.243M9.879 14.121l-4.243 4.243m0-10.606l4.243 4.243m4.242 4.242l4.243 4.243"/>
            </svg>
            <span>快捷键设置</span>
          </div>
          <div
            class="menu-item"
            :class="{ active: selectedMenu === 'data' }"
            @click="selectMenu('data')"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="7" height="7"/>
              <rect x="14" y="3" width="7" height="7"/>
              <rect x="14" y="14" width="7" height="7"/>
              <rect x="3" y="14" width="7" height="7"/>
            </svg>
            <span>我的数据</span>
          </div>
          <div
            class="menu-item"
            :class="{ active: selectedMenu === 'update' }"
            @click="selectMenu('update')"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 12a9 9 0 1 1-2.64-6.36"/>
              <path d="M21 3v6h-6"/>
              <path d="M12 7v6l4 2"/>
            </svg>
            <span>软件更新</span>
          </div>
          <div
            class="menu-item"
            :class="{ active: selectedMenu === 'ai' }"
            @click="selectMenu('ai')"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 2L2 7l10 5 10-5-10-5z"/>
              <path d="M2 17l10 5 10-5"/>
              <path d="M2 12l10 5 10-5"/>
            </svg>
            <span>AI 模型</span>
          </div>
          <div
            class="menu-item"
            :class="{ active: selectedMenu === 'mcp' }"
            @click="selectMenu('mcp')"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
              <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
              <line x1="12" y1="22.08" x2="12" y2="12"/>
            </svg>
            <span>MCP 配置</span>
          </div>
          <div
            class="menu-item"
            :class="{ active: selectedMenu === 'plugins' }"
            @click="selectMenu('plugins')"
          >
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="7" height="7"/>
              <rect x="14" y="3" width="7" height="7"/>
              <rect x="14" y="14" width="7" height="7"/>
              <rect x="3" y="14" width="7" height="7"/>
            </svg>
            <span>插件</span>
          </div>
        </div>
      </div>

      <!-- 右侧内容区 -->
      <div class="settings-content">
        <!-- 设置面板 -->
        <div v-if="selectedMenu === 'settings'" class="content-panel">
          <h2 class="panel-title">快捷键设置</h2>

          <!-- 搜索框快捷键 -->
          <div class="setting-group">
            <div class="setting-label">
              <span class="label-text">全局唤醒快捷键</span>
            </div>
            <div class="setting-control">
              <div v-if="!isEditingShortcut" class="shortcut-display">
                <kbd class="shortcut-key">{{ currentShortcut }}</kbd>
                <button class="btn-edit" @click="startEditingShortcut">修改</button>
              </div>
              <div v-else class="shortcut-edit">
                <div
                  class="shortcut-input"
                  tabindex="0"
                  @keydown="handleShortcutInput"
                >
                  <span v-if="!editingShortcut" class="placeholder">按下快捷键组合...</span>
                  <span v-else class="shortcut-text">{{ editingShortcut }}</span>
                </div>
                <div class="edit-actions">
                  <button class="btn-save" @click="saveShortcut">保存</button>
                  <button class="btn-cancel" @click="cancelEditingShortcut">取消</button>
                </div>
              </div>
              <div v-if="shortcutError" class="error-message">{{ shortcutError }}</div>
              <div v-if="shortcutSuccess" class="success-message">{{ shortcutSuccess }}</div>
            </div>
          </div>

          <div class="setting-info">
            <p>快捷键用于在任何地方快速唤醒应用</p>
            <p>建议使用 Alt+Space 或 Cmd+Space 等组合键</p>
          </div>
        </div>

        <!-- 其他菜单的占位内容 -->
        <div v-else-if="selectedMenu === 'data'" class="content-panel">
          <h2 class="panel-title">我的数据</h2>
          
          <!-- 数据统计卡片 -->
          <div class="data-stats-grid">
            <!-- 剪贴板统计 -->
            <div class="data-card">
              <div class="data-card-header">
                <div class="data-icon" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%)">
                  📋
                </div>
                <div class="data-title">剪贴板历史</div>
              </div>
              <div class="data-stats">
                <div class="stat-item">
                  <div class="stat-label">记录数量</div>
                  <div class="stat-value">{{ clipboardCount }} 条</div>
                </div>
                <div class="stat-item">
                  <div class="stat-label">占用空间</div>
                  <div class="stat-value">{{ clipboardSize }}</div>
                </div>
              </div>
              <div class="data-actions">
                <button class="btn-action btn-danger" @click="clearClipboardHistory">
                  清空历史
                </button>
              </div>
            </div>

            <!-- 备忘录统计 -->
            <div class="data-card">
              <div class="data-card-header">
                <div class="data-icon" style="background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%)">
                  📝
                </div>
                <div class="data-title">备忘快贴</div>
              </div>
              <div class="data-stats">
                <div class="stat-item">
                  <div class="stat-label">备忘数量</div>
                  <div class="stat-value">{{ memoCount }} 条</div>
                </div>
                <div class="stat-item">
                  <div class="stat-label">占用空间</div>
                  <div class="stat-value">{{ memoSize }}</div>
                </div>
              </div>
              <div class="data-actions">
                <button class="btn-action" disabled>
                  导出数据
                </button>
              </div>
            </div>
          </div>

          <!-- 数据说明 -->
          <div class="data-info">
            <h3 class="info-title">数据存储说明</h3>
            <ul class="info-list">
              <li>所有数据存储在本地，不会上传到云端</li>
              <li>剪贴板历史最多保存 100 条记录</li>
              <li>数据文件位置：~/.local/share/com.lewen.my-utools/</li>
              <li>可以手动备份数据文件以防数据丢失</li>
            </ul>
          </div>
        </div>

        <!-- AI 模型配置面板 -->
        <div v-else-if="selectedMenu === 'ai'" class="content-panel">
          <AiModelSettings />
        </div>

        <!-- MCP 配置面板 -->
        <div v-else-if="selectedMenu === 'mcp'" class="content-panel">
          <McpSettings />
        </div>

        <!-- 插件管理面板 -->
        <div v-else-if="selectedMenu === 'plugins'" class="content-panel">
          <div class="mcp-header">
            <div class="header-info">
              <h2 class="panel-title">插件管理</h2>
              <p class="panel-subtitle">
                管理已安装的插件，启用或禁用功能
              </p>
            </div>
          </div>

          <!-- 统计卡片 -->
          <div class="mcp-stats">
            <div class="stat-card">
              <div class="stat-value">2</div>
              <div class="stat-label">总插件数</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">2</div>
              <div class="stat-label">已启用</div>
            </div>
            <div class="stat-card">
              <div class="stat-value">0</div>
              <div class="stat-label">已禁用</div>
            </div>
          </div>

          <!-- 插件列表 -->
          <div class="server-list">
            <div class="server-item">
              <div class="server-header">
                <div class="plugin-icon" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%)">
                  📋
                </div>
                <div class="server-info">
                  <div class="server-name">
                    <span class="status-dot active"></span>
                    剪贴板历史
                  </div>
                  <div class="server-command">管理和搜索剪贴板历史记录</div>
                </div>
                <div class="server-actions">
                  <button class="btn-icon" title="已启用">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="20 6 9 17 4 12"/>
                    </svg>
                  </button>
                </div>
              </div>
            </div>

            <div class="server-item">
              <div class="server-header">
                <div class="plugin-icon" style="background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%)">
                  📝
                </div>
                <div class="server-info">
                  <div class="server-name">
                    <span class="status-dot active"></span>
                    备忘快贴
                  </div>
                  <div class="server-command">快速保存和粘贴常用文本</div>
                </div>
                <div class="server-actions">
                  <button class="btn-icon" title="已启用">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="20 6 9 17 4 12"/>
                    </svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 软件更新面板 -->
        <div v-else-if="selectedMenu === 'update'" class="content-panel">
          <h2 class="panel-title">软件更新</h2>

          <div class="update-card">
            <div class="update-version-row">
              <span class="update-current">当前版本</span>
              <span class="update-version-badge">v{{ appVersion || "…" }}</span>
            </div>

            <!-- 检查按钮 / 状态 -->
            <div class="update-status-row">
              <button
                class="update-check-btn"
                :disabled="updateStatus === 'checking' || updateStatus === 'downloading'"
                @click="checkForUpdate"
              >
                {{ updateStatus === 'checking' ? '检查中…' : '检查更新' }}
              </button>

              <span v-if="updateStatus === 'uptodate'" class="update-hint ok">已是最新版本 ✓</span>
              <span v-else-if="updateStatus === 'checking'" class="update-hint">正在检查…</span>
            </div>

            <!-- 发现新版本 -->
            <div v-if="updateStatus === 'available' || updateStatus === 'downloading' || updateStatus === 'ready'" class="update-available">
              <div class="update-new-version">
                发现新版本 <b>v{{ newVersion }}</b>
                <span class="update-size" v-if="totalBytes > 0">（约 {{ formatBytes(totalBytes) }}）</span>
              </div>
              <pre v-if="updateNotes" class="update-notes">{{ updateNotes }}</pre>

              <div v-if="updateStatus === 'available'" class="update-actions">
                <button class="update-install-btn" @click="downloadAndInstallUpdate">
                  下载并安装
                </button>
              </div>

              <!-- 下载进度 -->
              <div v-else-if="updateStatus === 'downloading'" class="update-progress">
                <div class="progress-bar">
                  <div class="progress-fill" :style="{ width: downloadProgress + '%' }"></div>
                </div>
                <span class="progress-text">
                  {{ downloadProgress }}%
                  <template v-if="totalBytes > 0">（{{ formatBytes(downloadedBytes) }} / {{ formatBytes(totalBytes) }}）</template>
                </span>
              </div>

              <!-- 安装完成 -->
              <div v-else-if="updateStatus === 'ready'" class="update-ready">
                <span>更新已安装，重启后生效</span>
                <button class="update-install-btn" @click="restartApp">立即重启</button>
              </div>
            </div>

            <!-- 错误 -->
            <div v-if="updateStatus === 'error'" class="update-error">
              <p>检查更新失败：{{ updateError }}</p>
              <p class="update-error-hint">尚未发布任何版本时更新源不存在，发布首个 Release 后即可正常检查。</p>
            </div>
          </div>
        </div>

        <!-- 其他菜单的占位内容 -->
        <div v-else class="content-panel">
          <h2 class="panel-title">{{ selectedMenu }}</h2>
          <p class="placeholder-text">此功能正在开发中...</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-container {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #2b2b2b;
}

/* 顶部标题栏 */
.settings-header {
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

.header-title {
  font-size: 14px;
  font-weight: 500;
}

.close-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: transparent;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-sm);
  color: #999;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.close-btn:hover {
  background: #4a4a4a;
  color: #e0e0e0;
  border-color: #5a5a5a;
}

/* 主体区域 */
.settings-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* 左侧导航 */
.settings-sidebar {
  width: 240px;
  background: #323232;
  border-right: 1px solid #4a4a4a;
  overflow-y: auto;
  padding: 16px 0;
}

.sidebar-section {
  margin-bottom: 24px;
}

.section-title {
  padding: 8px 20px;
  font-size: 12px;
  color: #888;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  color: #b0b0b0;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.menu-item:hover {
  background: #3a3a3a;
  color: #e0e0e0;
}

.menu-item.active {
  background: #3a3a3a;
  color: #e0e0e0;
}

.menu-item.active-highlight.active {
  background: #4a4a4a;
  color: #5a9fd4;
}

.menu-item.active-highlight.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: #5a9fd4;
}

/* 右侧内容区 */
.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 32px 40px;
}

.content-panel {
  max-width: 800px;
}

.panel-title {
  font-size: 16px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0 0 24px 0;
}

.placeholder-text {
  color: #888;
  font-size: 14px;
}

/* 设置项 */
.setting-group {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 0;
  border-bottom: 1px solid #3a3a3a;
}

.setting-group:last-child {
  border-bottom: none;
}

.setting-label {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.label-text {
  font-size: 14px;
  color: #e0e0e0;
}

.label-hint {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #4a4a4a;
  border-radius: 50%;
  font-size: 11px;
  color: #888;
  cursor: help;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* 快捷键显示 */
.shortcut-display {
  display: flex;
  align-items: center;
  gap: 12px;
}

.shortcut-key {
  display: inline-block;
  padding: 8px 16px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-sm);
  font-family: "SF Mono", "Monaco", monospace;
  font-size: 13px;
  font-weight: 500;
  color: #5a9fd4;
  min-width: 120px;
  text-align: center;
}

.btn-edit {
  padding: 8px 16px;
  background: transparent;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-sm);
  color: #e0e0e0;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-edit:hover {
  background: #4a4a4a;
  border-color: #5a5a5a;
}

/* 快捷键编辑 */
.shortcut-edit {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 300px;
}

.shortcut-input {
  padding: 12px 16px;
  background: #3a3a3a;
  border: 2px solid #5a9fd4;
  border-radius: var(--r-sm);
  min-height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  outline: none;
  cursor: text;
}

.shortcut-input:focus {
  box-shadow: 0 0 0 3px rgba(90, 159, 212, 0.2);
}

.shortcut-input .placeholder {
  color: #888;
  font-size: 12px;
}

.shortcut-text {
  font-family: "SF Mono", "Monaco", monospace;
  font-size: 14px;
  font-weight: 500;
  color: #5a9fd4;
}

.edit-actions {
  display: flex;
  gap: 8px;
}

.btn-save,
.btn-cancel {
  flex: 1;
  padding: 10px 16px;
  border-radius: var(--r-sm);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition: all 0.2s ease;
}

.btn-save {
  background: #5a9fd4;
  color: #1a1a1a;
}

.btn-save:hover {
  background: #6aafed;
}

.btn-cancel {
  background: #3a3a3a;
  color: #e0e0e0;
  border: 1px solid #4a4a4a;
}

.btn-cancel:hover {
  background: #4a4a4a;
}

/* 消息提示 */
.error-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(243, 139, 168, 0.1);
  border: 1px solid rgba(243, 139, 168, 0.3);
  border-radius: var(--r-sm);
  font-size: 12px;
  color: #f38ba8;
}

.success-message {
  margin-top: 8px;
  padding: 8px 12px;
  background: rgba(166, 227, 161, 0.1);
  border: 1px solid rgba(166, 227, 161, 0.3);
  border-radius: var(--r-sm);
  font-size: 12px;
  color: #a6e3a1;
}

/* 开关按钮 */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 26px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #4a4a4a;
  transition: 0.3s;
  border-radius: 999px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 20px;
  width: 20px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .toggle-slider {
  background-color: #5a9fd4;
}

input:checked + .toggle-slider:before {
  transform: translateX(22px);
}

/* 下拉选择框 */
.select-input {
  padding: 8px 12px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-sm);
  color: #e0e0e0;
  font-size: 13px;
  cursor: pointer;
  outline: none;
  min-width: 160px;
}

.select-input:hover {
  border-color: #5a5a5a;
}

/* 滑块 */
.range-input {
  width: 200px;
  height: 4px;
  background: #4a4a4a;
  border-radius: 2px;
  outline: none;
  -webkit-appearance: none;
}

.range-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 16px;
  height: 16px;
  background: #5a9fd4;
  border-radius: 50%;
  cursor: pointer;
}

.range-input::-moz-range-thumb {
  width: 16px;
  height: 16px;
  background: #5a9fd4;
  border-radius: 50%;
  cursor: pointer;
  border: none;
}

.range-value {
  font-size: 13px;
  color: #888;
  min-width: 80px;
}

/* 分隔线 */
.divider {
  height: 1px;
  background: #3a3a3a;
  margin: 32px 0;
}

/* 滚动条 */
.settings-sidebar::-webkit-scrollbar,
.settings-content::-webkit-scrollbar {
  width: 6px;
}

.settings-sidebar::-webkit-scrollbar-track,
.settings-content::-webkit-scrollbar-track {
  background: transparent;
}

.settings-sidebar::-webkit-scrollbar-thumb,
.settings-content::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 3px;
}

.settings-sidebar::-webkit-scrollbar-thumb:hover,
.settings-content::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}

/* 数据统计样式 */
.data-stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
  margin-bottom: 32px;
}

.data-card {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-card);
  padding: 24px;
  transition: all 0.2s ease;
}

.data-card:hover {
  border-color: #5a5a5a;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.data-card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.data-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  border-radius: var(--r-ctl);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.data-title {
  font-size: 16px;
  font-weight: 600;
  color: #e0e0e0;
}

.data-stats {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 20px;
  padding: 16px 0;
  border-top: 1px solid #3a3a3a;
  border-bottom: 1px solid #3a3a3a;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-label {
  font-size: 13px;
  color: #888;
}

.stat-value {
  font-size: 18px;
  font-weight: 600;
  color: #5a9fd4;
}

.data-actions {
  display: flex;
  gap: 12px;
}

.btn-action {
  flex: 1;
  padding: 10px 16px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-ctl);
  color: #e0e0e0;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-action:hover:not(:disabled) {
  background: #4a4a4a;
  border-color: #5a5a5a;
}

.btn-action:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-action.btn-danger {
  background: rgba(243, 139, 168, 0.1);
  border-color: rgba(243, 139, 168, 0.3);
  color: #f38ba8;
}

.btn-action.btn-danger:hover {
  background: rgba(243, 139, 168, 0.2);
  border-color: rgba(243, 139, 168, 0.5);
}

.data-info {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-card);
  padding: 24px;
}

.info-title {
  font-size: 14px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0 0 16px 0;
}

.info-list {
  margin: 0;
  padding-left: 20px;
  color: #888;
  font-size: 13px;
  line-height: 1.8;
}

.info-list li {
  margin-bottom: 8px;
}

.setting-info {
  margin-top: 24px;
  padding: 16px;
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-ctl);
}

.setting-info p {
  margin: 0 0 8px 0;
  font-size: 13px;
  color: #888;
  line-height: 1.6;
}

.setting-info p:last-child {
  margin-bottom: 0;
}

/* 插件图标 */
.plugin-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--r-card);
  font-size: 24px;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

/* MCP 样式（用于插件页面） */
.mcp-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.header-info {
  flex: 1;
}

.mcp-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-value {
  font-size: 32px;
  font-weight: 700;
  color: #5a9fd4;
  margin-bottom: 8px;
}

.stat-label {
  font-size: 12px;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.server-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.server-item {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-ctl);
  overflow: hidden;
  transition: all 0.2s ease;
}

.server-item:hover {
  border-color: #5a5a5a;
}

.server-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  gap: 16px;
}

.server-info {
  flex: 1;
}

.server-name {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  font-weight: 600;
  color: #e0e0e0;
  margin-bottom: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #666;
}

.status-dot.active {
  background: #4ade80;
  box-shadow: 0 0 8px rgba(74, 222, 128, 0.5);
}

.server-command {
  font-size: 12px;
  color: #888;
}

.server-actions {
  display: flex;
  gap: 8px;
}

.btn-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid #4a4a4a;
  border-radius: var(--r-sm);
  color: #b0b0b0;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-icon:hover {
  background: #3a3a3a;
  border-color: #5a5a5a;
  color: #e0e0e0;
}

/* ===== 软件更新 ===== */
.update-card {
  padding: 20px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--r-card);
}

.update-version-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.update-current {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.6);
}

.update-version-badge {
  padding: 3px 10px;
  font-size: 12px;
  font-weight: 600;
  color: #0a84ff;
  background: rgba(10, 132, 255, 0.12);
  border-radius: var(--r-ctl);
}

.update-status-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.update-check-btn {
  padding: 7px 16px;
  font-size: 12px;
  color: #fff;
  background: linear-gradient(135deg, #0a84ff, #0066d6);
  border: none;
  border-radius: var(--r-ctl);
  cursor: pointer;
}

.update-check-btn:hover:not(:disabled) {
  filter: brightness(1.12);
}

.update-check-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.update-hint {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.5);
}

.update-hint.ok {
  color: #30d158;
}

.update-available {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.update-new-version {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
}

.update-new-version b {
  color: #4fe0c8;
}

.update-size {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.45);
}

.update-notes {
  max-height: 140px;
  overflow-y: auto;
  margin: 12px 0;
  padding: 12px;
  font-size: 12px;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.7);
  background: rgba(0, 0, 0, 0.25);
  border-radius: var(--r-ctl);
  white-space: pre-wrap;
  font-family: inherit;
}

.update-actions {
  display: flex;
  gap: 10px;
}

.update-install-btn {
  padding: 8px 20px;
  font-size: 13px;
  color: #fff;
  background: linear-gradient(135deg, #30d158, #248a3d);
  border: none;
  border-radius: var(--r-ctl);
  cursor: pointer;
}

.update-install-btn:hover {
  filter: brightness(1.1);
}

.update-progress {
  display: flex;
  align-items: center;
  gap: 12px;
}

.progress-bar {
  flex: 1;
  height: 8px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #0a84ff, #30d158);
  border-radius: 4px;
  transition: width 200ms ease;
}

.progress-text {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.6);
  white-space: nowrap;
}

.update-ready {
  display: flex;
  align-items: center;
  gap: 14px;
  font-size: 13px;
  color: #30d158;
}

.update-error {
  margin-top: 14px;
  padding: 12px 14px;
  background: rgba(255, 82, 82, 0.1);
  border: 1px solid rgba(255, 82, 82, 0.25);
  border-radius: var(--r-ctl);
}

.update-error p {
  margin: 0 0 4px;
  font-size: 12px;
  color: #ff8787;
  word-break: break-all;
}

.update-error-hint {
  color: rgba(255, 255, 255, 0.45) !important;
}
</style>
