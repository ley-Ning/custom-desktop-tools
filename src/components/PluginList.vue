<script setup lang="ts">
import type { App } from "../types";
import AppIcon from "./AppIcon.vue";

const props = defineProps<{
  apps: App[];
  selectedIndex: number;
}>();

const emit = defineEmits<{
  select: [app: App];
}>();
</script>

<template>
  <div class="search-results">
    <div
      v-for="(app, index) in apps"
      :key="app.id"
      class="result-item"
      :class="{ selected: index === selectedIndex }"
      @click="emit('select', app)"
    >
      <!-- App Icon -->
      <div class="result-icon">
        <AppIcon :icon="app.icon" :size="40" />
      </div>

      <!-- App Info -->
      <div class="result-info">
        <div class="result-name">{{ app.name }}</div>
        <div class="result-path">{{ app.path }}</div>
      </div>

      <!-- Enter 提示 -->
      <div v-if="index === selectedIndex" class="enter-hint">
        <kbd>↵</kbd>
      </div>
    </div>
  </div>
</template>

<style scoped>
.search-results {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
  background: #2b2b2b;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  margin-bottom: 4px;
}

.result-item:hover,
.result-item.selected {
  background: #3a3a3a;
}

.result-item.selected {
  box-shadow: 0 0 0 2px #5a9fd4 inset;
}

.result-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: 8px;
  overflow: hidden;
  background: #3a3a3a;
}

.result-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.result-name {
  font-size: 14px;
  font-weight: 500;
  color: #e0e0e0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-path {
  font-size: 11px;
  color: #888;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.enter-hint {
  flex-shrink: 0;
}

.enter-hint kbd {
  display: inline-block;
  padding: 4px 8px;
  background: #4a4a4a;
  border: 1px solid #5a5a5a;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  color: #e0e0e0;
  font-family: monospace;
}

/* 滚动条 */
.search-results::-webkit-scrollbar {
  width: 6px;
}

.search-results::-webkit-scrollbar-track {
  background: transparent;
}

.search-results::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 3px;
}

.search-results::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}
</style>
