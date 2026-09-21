<script setup lang="ts">
import type { App } from "../types";
import AppIcon from "./AppIcon.vue";

defineProps<{
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
  padding: 10px 12px;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 14px;
  border-radius: var(--r-ctl);
  border: 1px solid transparent;
  cursor: pointer;
  transition: background 140ms ease, border-color 140ms ease, transform 140ms ease;
  margin-bottom: 2px;
  animation: rowIn 200ms cubic-bezier(0.25, 0.1, 0.25, 1) backwards;
}

@keyframes rowIn {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.result-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.result-item.selected {
  background: linear-gradient(90deg, rgba(10, 132, 255, 0.2), rgba(125, 92, 255, 0.12));
  border-color: rgba(10, 132, 255, 0.35);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
}

.result-item:active {
  transform: scale(0.995);
}

.result-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  border-radius: var(--r-ctl);
  overflow: hidden;
  background: rgba(255, 255, 255, 0.06);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.result-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.result-name {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.92);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.result-path {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.38);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.enter-hint {
  flex-shrink: 0;
  opacity: 0;
  animation: fadeIn 160ms ease 60ms forwards;
}

@keyframes fadeIn {
  to {
    opacity: 1;
  }
}

.enter-hint kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 26px;
  height: 24px;
  padding: 0 6px;
  background: rgba(10, 132, 255, 0.22);
  border: 1px solid rgba(10, 132, 255, 0.5);
  border-radius: var(--r-sm);
  font-size: 12px;
  font-weight: 600;
  color: #7cc0ff;
  font-family: inherit;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
}

/* 滚动条 */
.search-results::-webkit-scrollbar {
  width: 6px;
}

.search-results::-webkit-scrollbar-track {
  background: transparent;
}

.search-results::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.14);
  border-radius: 3px;
}

.search-results::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.24);
}
</style>
