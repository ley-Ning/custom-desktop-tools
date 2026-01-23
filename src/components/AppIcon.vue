<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  icon?: string;
  size?: number;
}>();

// 检查是否为 base64 图标数据
const isBase64Icon = computed(() => {
  return props.icon && props.icon.startsWith("data:image");
});

// 默认大小
const iconSize = computed(() => props.size || 32);
</script>

<template>
  <!-- 如果是 base64 图标，显示图片 -->
  <img
    v-if="isBase64Icon"
    :src="icon"
    :width="iconSize"
    :height="iconSize"
    class="app-icon-img"
    alt="App icon"
  />
  <!-- 否则显示默认 emoji -->
  <span v-else class="fallback-icon">📱</span>
</template>

<style scoped>
.app-icon-img {
  display: block;
  border-radius: 22.5%; /* iOS 风格圆角 */
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15),
              0 1px 3px rgba(0, 0, 0, 0.1);
  transition: transform 0.2s ease;
}

.app-icon-img:hover {
  transform: scale(1.05);
}

.fallback-icon {
  font-size: v-bind("iconSize + 'px'");
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: v-bind("iconSize + 'px'");
  height: v-bind("iconSize + 'px'");
  border-radius: 22.5%;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}
</style>
