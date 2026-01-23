<script setup lang="ts">
import { ref, computed } from "vue";

const emit = defineEmits<{
  close: [];
}>();

// 搜索关键词
const searchQuery = ref("");

// 已安装的插件
const installedPlugins = ref([
  { id: "clipboard", name: "剪贴板历史", icon: "📋", color: "linear-gradient(135deg, #667eea 0%, #764ba2 100%)", description: "管理和搜索剪贴板历史记录" },
  { id: "memo", name: "备忘快贴", icon: "📝", color: "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)", description: "快速保存和粘贴常用文本" },
]);

// 过滤后的插件
const filteredPlugins = computed(() => {
  if (!searchQuery.value) return installedPlugins.value;
  return installedPlugins.value.filter((plugin) =>
    plugin.name.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
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
        <button class="tab-btn active">插件应用市场</button>
        <button class="tab-btn close-btn" @click="emit('close')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="market-body">
      <!-- 主内容区 -->
      <div class="market-content">
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
            placeholder="搜索已安装的插件..."
          />
        </div>

        <!-- 已安装插件列表 -->
        <div class="section">
          <div class="section-header">
            <h3>已安装插件 ({{ installedPlugins.length }})</h3>
          </div>

          <div v-if="filteredPlugins.length > 0" class="plugins-grid">
            <div
              v-for="plugin in filteredPlugins"
              :key="plugin.id"
              class="plugin-card"
            >
              <div class="plugin-icon-large" :style="{ background: plugin.color }">
                {{ plugin.icon }}
              </div>
              <div class="plugin-info">
                <h4>{{ plugin.name }}</h4>
                <p>{{ plugin.description }}</p>
              </div>
            </div>
          </div>

          <div v-else class="empty-state">
            <div class="empty-icon">🔍</div>
            <p>未找到匹配的插件</p>
          </div>
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
  padding: 6px 16px;
  background: transparent;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  color: #e0e0e0;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
  -webkit-app-region: no-drag;
}

.tab-btn.active {
  background: #4a4a4a;
  border-color: #5a5a5a;
}

.tab-btn.close-btn {
  padding: 6px 10px;
  min-width: auto;
}

.tab-btn:hover {
  background: #4a4a4a;
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
  margin-bottom: 20px;
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



/* 区块 */
.section {
  margin-bottom: 40px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.section-header h3 {
  font-size: 20px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0;
}

/* 插件网格 */
.plugins-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.plugin-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 32px 24px;
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 12px;
  transition: all 0.2s ease;
}

.plugin-card:hover {
  background: #3a3a3a;
  border-color: #5a5a5a;
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.plugin-icon-large {
  width: 80px;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 16px;
  font-size: 48px;
  flex-shrink: 0;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
}

.plugin-info {
  text-align: center;
  width: 100%;
}

.plugin-info h4 {
  font-size: 16px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0 0 8px 0;
}

.plugin-info p {
  font-size: 13px;
  color: #888;
  margin: 0;
  line-height: 1.5;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-state p {
  font-size: 14px;
  color: #888;
  margin: 0;
}

/* 滚动条 */
.market-content::-webkit-scrollbar {
  width: 8px;
}

.market-content::-webkit-scrollbar-track {
  background: transparent;
}

.market-content::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 4px;
}

.market-content::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}
</style>
