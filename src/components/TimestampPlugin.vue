<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import dayjs from 'dayjs';
import utc from 'dayjs/plugin/utc';
import timezone from 'dayjs/plugin/timezone';
import relativeTime from 'dayjs/plugin/relativeTime';
import quarterOfYear from 'dayjs/plugin/quarterOfYear';
import weekOfYear from 'dayjs/plugin/weekOfYear';
import dayOfYear from 'dayjs/plugin/dayOfYear';
import 'dayjs/locale/zh-cn';

// 配置 dayjs
dayjs.extend(utc);
dayjs.extend(timezone);
dayjs.extend(relativeTime);
dayjs.extend(quarterOfYear);
dayjs.extend(weekOfYear);
dayjs.extend(dayOfYear);
dayjs.locale('zh-cn');

const emit = defineEmits<{
  close: [];
}>();

const timestampInput = ref("");
const dateInput = ref("");
const currentTime = ref(Date.now());

// 更新当前时间
setInterval(() => {
  currentTime.value = Date.now();
}, 1000);

// 时间戳转日期
const timestampToDate = computed(() => {
  if (!timestampInput.value) return null;
  
  try {
    const ts = parseInt(timestampInput.value);
    if (isNaN(ts)) return { error: "无效的时间戳" };
    
    // 自动判断是秒还是毫秒
    const timestamp = ts.toString().length === 10 ? ts * 1000 : ts;
    const date = dayjs(timestamp);
    
    if (!date.isValid()) return { error: "无效的时间戳" };
    
    return {
      iso: date.toISOString(),
      local: date.format('YYYY-MM-DD HH:mm:ss'),
      utc: date.utc().format('YYYY-MM-DD HH:mm:ss [UTC]'),
      relative: date.fromNow(),
      year: date.year(),
      month: date.month() + 1,
      day: date.date(),
      hour: date.hour(),
      minute: date.minute(),
      second: date.second(),
      weekday: date.format('dddd'),
      quarter: date.quarter(),
      dayOfYear: date.dayOfYear(),
      week: date.week(),
    };
  } catch (e) {
    return { error: "转换失败" };
  }
});

// 日期转时间戳
const dateToTimestamp = computed(() => {
  if (!dateInput.value) return null;
  
  try {
    const date = dayjs(dateInput.value);
    if (!date.isValid()) return { error: "无效的日期" };
    
    return {
      seconds: Math.floor(date.valueOf() / 1000),
      milliseconds: date.valueOf(),
      iso: date.toISOString(),
      unix: date.unix(),
    };
  } catch (e) {
    return { error: "转换失败" };
  }
});

// 当前时间信息
const currentTimeInfo = computed(() => {
  const date = dayjs(currentTime.value);
  return {
    timestamp: Math.floor(currentTime.value / 1000),
    timestampMs: currentTime.value,
    iso: date.toISOString(),
    local: date.format('YYYY-MM-DD HH:mm:ss'),
    utc: date.utc().format('YYYY-MM-DD HH:mm:ss [UTC]'),
    weekday: date.format('dddd'),
    year: date.year(),
    month: date.month() + 1,
    day: date.date(),
  };
});

// 复制到剪贴板
async function copyToClipboard(text: string | number | undefined) {
  try {
    if (text === undefined) return;
    await invoke("write_clipboard_text", { text: text.toString() });
    // 简单的视觉反馈
    const btn = event?.target as HTMLElement;
    if (btn) {
      const originalText = btn.textContent;
      btn.textContent = '✓ 已复制';
      setTimeout(() => {
        btn.textContent = originalText;
      }, 1000);
    }
  } catch (e) {
    console.error("Failed to copy:", e);
    alert("复制失败");
  }
}

// 使用当前时间
function useCurrentTime() {
  timestampInput.value = Math.floor(currentTime.value / 1000).toString();
}

// 使用当前日期
function useCurrentDate() {
  dateInput.value = dayjs().format('YYYY-MM-DDTHH:mm');
}

// 常用时间快捷方式
function setQuickTime(type: string) {
  const now = dayjs();
  let date: dayjs.Dayjs;
  
  switch (type) {
    case 'now':
      date = now;
      break;
    case 'today-start':
      date = now.startOf('day');
      break;
    case 'today-end':
      date = now.endOf('day');
      break;
    case 'yesterday':
      date = now.subtract(1, 'day').startOf('day');
      break;
    case 'tomorrow':
      date = now.add(1, 'day').startOf('day');
      break;
    case 'week-start':
      date = now.startOf('week');
      break;
    case 'week-end':
      date = now.endOf('week');
      break;
    case 'month-start':
      date = now.startOf('month');
      break;
    case 'month-end':
      date = now.endOf('month');
      break;
    default:
      return;
  }
  
  timestampInput.value = date.unix().toString();
}
</script>

<template>
  <div class="timestamp-plugin">
    <!-- 顶部工具栏 -->
    <div class="plugin-header" data-tauri-drag-region>
      <div class="header-left">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <polyline points="12 6 12 12 16 14"/>
        </svg>
        <span class="plugin-title">时间戳转换</span>
      </div>
      <button class="close-btn" @click="emit('close')" title="关闭">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"/>
          <line x1="6" y1="6" x2="18" y2="18"/>
        </svg>
      </button>
    </div>

    <div class="plugin-content">
      <!-- 当前时间 -->
      <div class="section">
        <h3 class="section-title">当前时间</h3>
        <div class="time-card current-time">
          <div class="time-display">{{ currentTimeInfo.local }}</div>
          <div class="time-subtitle">{{ currentTimeInfo.weekday }}</div>
          <div class="time-info">
            <div class="info-row" @click="copyToClipboard(currentTimeInfo.timestamp)">
              <span class="label">时间戳（秒）</span>
              <span class="value clickable">{{ currentTimeInfo.timestamp }}</span>
            </div>
            <div class="info-row" @click="copyToClipboard(currentTimeInfo.timestampMs)">
              <span class="label">时间戳（毫秒）</span>
              <span class="value clickable">{{ currentTimeInfo.timestampMs }}</span>
            </div>
            <div class="info-row" @click="copyToClipboard(currentTimeInfo.iso)">
              <span class="label">ISO 8601</span>
              <span class="value clickable">{{ currentTimeInfo.iso }}</span>
            </div>
            <div class="info-row" @click="copyToClipboard(currentTimeInfo.utc)">
              <span class="label">UTC</span>
              <span class="value clickable">{{ currentTimeInfo.utc }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 快捷时间 -->
      <div class="section">
        <h3 class="section-title">快捷时间</h3>
        <div class="quick-buttons">
          <button class="quick-btn" @click="setQuickTime('now')">现在</button>
          <button class="quick-btn" @click="setQuickTime('today-start')">今天开始</button>
          <button class="quick-btn" @click="setQuickTime('today-end')">今天结束</button>
          <button class="quick-btn" @click="setQuickTime('yesterday')">昨天</button>
          <button class="quick-btn" @click="setQuickTime('tomorrow')">明天</button>
          <button class="quick-btn" @click="setQuickTime('week-start')">本周开始</button>
          <button class="quick-btn" @click="setQuickTime('week-end')">本周结束</button>
          <button class="quick-btn" @click="setQuickTime('month-start')">本月开始</button>
          <button class="quick-btn" @click="setQuickTime('month-end')">本月结束</button>
        </div>
      </div>

      <!-- 时间戳转日期 -->
      <div class="section">
        <h3 class="section-title">时间戳 → 日期</h3>
        <div class="input-group">
          <input
            v-model="timestampInput"
            type="text"
            class="input-field"
            placeholder="输入时间戳（支持秒/毫秒）"
          />
          <button class="btn-secondary" @click="useCurrentTime">当前</button>
        </div>
        
        <div v-if="timestampToDate && !timestampToDate.error" class="result-card">
          <div class="info-row" @click="copyToClipboard(timestampToDate.local)">
            <span class="label">本地时间</span>
            <span class="value clickable">{{ timestampToDate.local }}</span>
          </div>
          <div class="info-row" @click="copyToClipboard(timestampToDate.iso)">
            <span class="label">ISO 8601</span>
            <span class="value clickable">{{ timestampToDate.iso }}</span>
          </div>
          <div class="info-row" @click="copyToClipboard(timestampToDate.utc)">
            <span class="label">UTC</span>
            <span class="value clickable">{{ timestampToDate.utc }}</span>
          </div>
          <div class="info-row">
            <span class="label">相对时间</span>
            <span class="value">{{ timestampToDate.relative }}</span>
          </div>
          <div class="info-row">
            <span class="label">星期</span>
            <span class="value">{{ timestampToDate.weekday }}</span>
          </div>
          <div class="detail-grid">
            <div class="detail-item">
              <span class="detail-label">年</span>
              <span class="detail-value">{{ timestampToDate.year }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">月</span>
              <span class="detail-value">{{ timestampToDate.month }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">日</span>
              <span class="detail-value">{{ timestampToDate.day }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">时</span>
              <span class="detail-value">{{ timestampToDate.hour }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">分</span>
              <span class="detail-value">{{ timestampToDate.minute }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">秒</span>
              <span class="detail-value">{{ timestampToDate.second }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">季度</span>
              <span class="detail-value">Q{{ timestampToDate.quarter }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">第几周</span>
              <span class="detail-value">{{ timestampToDate.week }}</span>
            </div>
            <div class="detail-item">
              <span class="detail-label">第几天</span>
              <span class="detail-value">{{ timestampToDate.dayOfYear }}</span>
            </div>
          </div>
        </div>
        <div v-else-if="timestampToDate?.error" class="error-message">
          {{ timestampToDate.error }}
        </div>
      </div>

      <!-- 日期转时间戳 -->
      <div class="section">
        <h3 class="section-title">日期 → 时间戳</h3>
        <div class="input-group">
          <input
            v-model="dateInput"
            type="datetime-local"
            class="input-field"
          />
          <button class="btn-secondary" @click="useCurrentDate">当前</button>
        </div>
        
        <div v-if="dateToTimestamp && !dateToTimestamp.error" class="result-card">
          <div class="info-row" @click="copyToClipboard(dateToTimestamp.seconds)">
            <span class="label">时间戳（秒）</span>
            <span class="value clickable">{{ dateToTimestamp.seconds }}</span>
          </div>
          <div class="info-row" @click="copyToClipboard(dateToTimestamp.milliseconds)">
            <span class="label">时间戳（毫秒）</span>
            <span class="value clickable">{{ dateToTimestamp.milliseconds }}</span>
          </div>
          <div class="info-row" @click="copyToClipboard(dateToTimestamp.unix)">
            <span class="label">Unix 时间戳</span>
            <span class="value clickable">{{ dateToTimestamp.unix }}</span>
          </div>
          <div class="info-row" @click="copyToClipboard(dateToTimestamp.iso)">
            <span class="label">ISO 8601</span>
            <span class="value clickable">{{ dateToTimestamp.iso }}</span>
          </div>
        </div>
        <div v-else-if="dateToTimestamp?.error" class="error-message">
          {{ dateToTimestamp.error }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.timestamp-plugin {
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

.close-btn {
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

.close-btn:hover {
  background: #e74c3c;
  border-color: #e74c3c;
  color: #fff;
}

.plugin-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0 0 12px 0;
}

.time-card {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 8px;
  padding: 16px;
}

.current-time {
  border-color: #5a9fd4;
}

.time-display {
  font-size: 24px;
  font-weight: 600;
  color: #5a9fd4;
  text-align: center;
  margin-bottom: 4px;
}

.time-subtitle {
  font-size: 14px;
  color: #888;
  text-align: center;
  margin-bottom: 16px;
}

.time-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #3a3a3a;
  cursor: pointer;
  transition: background 0.2s ease;
}

.info-row:hover {
  background: rgba(90, 159, 212, 0.1);
}

.info-row:last-child {
  border-bottom: none;
}

.label {
  font-size: 12px;
  color: #888;
}

.value {
  font-size: 13px;
  color: #e0e0e0;
  font-family: monospace;
}

.value.clickable {
  color: #5a9fd4;
}

.quick-buttons {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.quick-btn {
  padding: 8px 12px;
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  color: #e0e0e0;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.quick-btn:hover {
  background: #4a4a4a;
  border-color: #5a9fd4;
  color: #5a9fd4;
}

.input-group {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.input-field {
  flex: 1;
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  padding: 10px 12px;
  font-size: 13px;
  color: #e0e0e0;
  outline: none;
  transition: all 0.2s ease;
}

.input-field:focus {
  border-color: #5a9fd4;
  box-shadow: 0 0 0 2px rgba(90, 159, 212, 0.2);
}

.btn-secondary {
  padding: 10px 16px;
  background: #4a4a4a;
  border: 1px solid #5a5a5a;
  border-radius: 6px;
  color: #e0e0e0;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.btn-secondary:hover {
  background: #5a5a5a;
  border-color: #6a6a6a;
}

.result-card {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 8px;
  padding: 12px;
}

.detail-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #3a3a3a;
}

.detail-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px;
  background: #2b2b2b;
  border-radius: 6px;
}

.detail-label {
  font-size: 11px;
  color: #888;
}

.detail-value {
  font-size: 16px;
  font-weight: 600;
  color: #5a9fd4;
}

.error-message {
  padding: 12px;
  background: rgba(231, 76, 60, 0.1);
  border: 1px solid #e74c3c;
  border-radius: 6px;
  color: #e74c3c;
  font-size: 13px;
}

.plugin-content::-webkit-scrollbar {
  width: 6px;
}

.plugin-content::-webkit-scrollbar-track {
  background: transparent;
}

.plugin-content::-webkit-scrollbar-thumb {
  background: #4a4a4a;
  border-radius: 3px;
}

.plugin-content::-webkit-scrollbar-thumb:hover {
  background: #5a5a5a;
}
</style>
