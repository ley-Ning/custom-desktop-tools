<script setup lang="ts">
import { ref, onMounted } from "vue";

const model = defineModel<string>({ required: true });
const inputRef = ref<HTMLInputElement>();

onMounted(() => {
  inputRef.value?.focus();
});
</script>

<template>
  <div class="search-bar">
    <div class="search-input-wrapper">
      <!-- SVG Search Icon -->
      <svg class="search-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
        <circle cx="11" cy="11" r="6" stroke="currentColor" stroke-width="2"/>
        <path d="M15.5 15.5L20 20" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
      </svg>
      <input
        ref="inputRef"
        v-model="model"
        type="text"
        placeholder="Search or type / for commands"
        class="search-input"
      />
    </div>
  </div>
</template>

<style scoped>
.search-bar {
  padding: 0;
}

.search-input-wrapper {
  display: flex;
  align-items: center;
  background: var(--bg-primary);
  border-radius: 12px;
  padding: 0 16px;
  height: 48px;
  transition: box-shadow 0.2s ease;
}

.search-input-wrapper:focus-within {
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.15);
}

.search-icon {
  width: 18px;
  height: 18px;
  margin-right: 12px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  font-size: 13px;
  font-weight: 400;
  color: var(--text-primary);
  font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", sans-serif;
  letter-spacing: -0.01em;
}

.search-input::placeholder {
  color: var(--text-secondary);
  opacity: 0.8;
}
</style>
