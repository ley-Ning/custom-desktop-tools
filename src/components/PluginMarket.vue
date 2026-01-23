<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { getAllPlugins, updatePlugin, type Plugin } from "../db";

const emit = defineEmits<{
  close: [];
}>();

// 搜索关键词
const searchQuery = ref("");

// 已安装的插件
const installedPlugins = ref<Plugin[]>([]);

// 加载插件列表
async function loadPlugins() {
  try {
    installedPlugins.value = await getAllPlugins();
  } catch (e) {
    console.error("Failed to load plugins:", e);
  }
}

// 切换插件启用状态
async function togglePlugin(plugin: Plugin) {
  try {
    plugin.enabled = !plugin.enabled;
    await updatePlugin(plugin);
  } catch (e) {
    console.error("Failed to toggle plugin:", e);
    // 回滚状态
    plugin.enabled = !plugin.enabled;
  }
}

// 过滤后的插件
const filteredPlugins = computed(() => {
  if (!searchQuery.value) return installedPlugins.value;
  return installedPlugins.value.filter((plugin) =>
    plugin.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
    plugin.description.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

onMounted(() => {
  loadPlugins();
});
</script>

<template>
  <div class="plugin-market">
    <!-- 顶部标题栏 -->
    <div class="market-header" data-tauri-drag-region>
      <div class="header-left">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2L2 7l10 5 10-5-10-5z"/>
          <path d="M2 17l10 5 10-5"/>
          <path d="M2 12l10 5 10-5"/>
        </svg>
        <span class="header-title">管理中心</span>
      </div>
      <div class="header-tabs">
        <button class="tab-btn close-btn" @click="emit('close')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
          <span>设置</span>
        </button>
      </div>
    </div>

    <div class="market-body">
      <!-- 主内容区 -->
      <div class="market-content">
        <!-- 标题和描述 -->
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
            <div class="stat-value">{{ installedPlugins.length }}</div>
            <div class="stat-label">总插件数</div>
          </div>
          <div class="stat-card">
            <div class="stat-value">{{ installedPlugins.filter(p => p.enabled).length }}</div>
            <div class="stat-label">已启用</div>
          </div>
          <div class="stat-card">
            <div class="stat-value">{{ installedPlugins.filter(p => !p.enabled).length }}</div>
            <div class="stat-label">已禁用</div>
          </div>
        </div>

        <!-- 搜索栏 -->
        <div class="search-bar">
          <svg class="search-icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.35-4.35"/>
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            class="search-input"
            placeholder="搜索插件..."
          />
        </div>

        <!-- 插件列表 -->
        <div class="server-list">
          <div
            v-for="plugin in filteredPlugins"
            :key="plugin.id"
            class="server-item"
            :class="{ disabled: !plugin.enabled }"
          >
            <div class="server-header">
              <div class="plugin-icon" :style="{ background: plugin.gradient }">
                {{ plugin.icon }}
              </div>
              <div class="server-info">
                <div class="server-name">
                  <span class="status-dot" :class="{ active: plugin.enabled }"></span>
                  {{ plugin.name }}
                  <span v-if="plugin.builtin" class="builtin-badge">内置</span>
                </div>
                <div class="server-command">{{ plugin.description }}</div>
                <div v-if="plugin.version" class="plugin-meta">
                  <span class="meta-item">v{{ plugin.version }}</span>
                  <span v-if="plugin.author" class="meta-item">{{ plugin.author }}</span>
                </div>
              </div>
              <div class="server-actions">
                <button
                  class="btn-icon"
                  :class="{ active: plugin.enabled }"
                  :title="plugin.enabled ? '禁用' : '启用'"
                  @click="togglePlugin(plugin)"
                >
                  <svg v-if="!plugin.enabled" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"/>
                    <line x1="15" y1="9" x2="9" y2="15"/>
                    <line x1="9" y1="9" x2="15" y2="15"/>
                  </svg>
                  <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="20 6 9 17 4 12"/>
                  </svg>
                </button>
              </div>
            </div>
          </div>
        </div>

        <div v-if="filteredPlugins.length === 0" class="empty-state">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="7" height="7"/>
            <rect x="14" y="3" width="7" height="7"/>
            <rect x="14" y="14" width="7" height="7"/>
            <rect x="3" y="14" width="7" height="7"/>
          </svg>
          <p>未找到匹配的插件</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.plugin-market {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #2b2b2b;
}

/* 顶部标题栏 */
.market-header {
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

.header-tabs {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: transparent;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  color: #999;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.tab-btn:hover {
  background: #4a4a4a;
  color: #e0e0e0;
  border-color: #5a5a5a;
}

/* 主体区域 */
.market-body {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* 主内容区 */
.market-content {
  flex: 1;
  overflow-y: auto;
  padding: 32px 48px;
  max-width: 900px;
  margin: 0 auto;
  width: 100%;
}

/* 标题区域 */
.mcp-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.header-info {
  flex: 1;
}

.panel-title {
  font-size: 20px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0 0 8px 0;
}

.panel-subtitle {
  font-size: 13px;
  color: #888;
  margin: 0;
}

/* 统计卡片 */
.mcp-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 10px;
  padding: 20px;
  text-align: center;
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

/* 搜索栏 */
.search-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 8px;
  margin-bottom: 24px;
}

.search-icon {
  color: #888;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 14px;
  color: #e0e0e0;
}

.search-input::placeholder {
  color: #666;
}

/* 服务器列表 */
.server-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

.server-item {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 10px;
  overflow: hidden;
  transition: all 0.2s ease;
}

.server-item:hover {
  border-color: #5a5a5a;
}

.server-item.disabled {
  opacity: 0.6;
}

.server-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  gap: 16px;
}

.plugin-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  font-size: 24px;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
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

.builtin-badge {
  display: inline-block;
  padding: 2px 8px;
  background: rgba(90, 159, 212, 0.2);
  border: 1px solid rgba(90, 159, 212, 0.3);
  border-radius: 4px;
  font-size: 10px;
  color: #5a9fd4;
  font-weight: 500;
  margin-left: 8px;
}

.plugin-meta {
  display: flex;
  gap: 12px;
  margin-top: 4px;
}

.meta-item {
  font-size: 11px;
  color: #666;
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
  border-radius: 6px;
  color: #b0b0b0;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-icon:hover {
  background: #3a3a3a;
  border-color: #5a5a5a;
  color: #e0e0e0;
}

.btn-icon.active {
  background: rgba(74, 222, 128, 0.1);
  border-color: #4ade80;
  color: #4ade80;
}

.btn-icon.active:hover {
  background: rgba(74, 222, 128, 0.2);
}

/* 空状态 */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #888;
}

.empty-state svg {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-state p {
  margin: 8px 0;
  font-size: 14px;
}

/* 滚动条 */
.market-content::-webkit-scrollbar {
  width: 6px;
}

.market-content::-webkit-scrollbar-track {
  background: transparent;
}

.market-content::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 3px;
}

.market-content::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}
</style>
