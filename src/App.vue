<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import PluginList from "./components/PluginList.vue";
import Settings from "./components/Settings.vue";
import PluginMarket from "./components/PluginMarket.vue";
import ClipboardPlugin from "./components/ClipboardPlugin.vue";
import MemoPlugin from "./components/MemoPlugin.vue";
import JsonEditorPlugin from "./components/JsonEditorPlugin.vue";
import TimestampPlugin from "./components/TimestampPlugin.vue";
import AiChatPlugin from "./components/AiChatPlugin.vue";
import TranslatorPlugin from "./components/TranslatorPlugin.vue";
import CalcPlugin from "./components/CalcPlugin.vue";
import TextToolPlugin from "./components/TextToolPlugin.vue";
import type { App, Plugin } from "./types";
import { initDB, getEnabledPlugins, getClipboardHistory, getMemos } from "./db";

// 插件定义（从 IndexedDB 加载）
const allPlugins = ref<Plugin[]>([]);

const searchQuery = ref("");
const apps = ref<App[]>([]);
const recentApps = ref<App[]>([]);
const selectedIndex = ref(0);
const searchInput = ref<HTMLInputElement | null>(null);
const showSettings = ref(false);
const showPluginMarket = ref(false);
const showClipboard = ref(false);
const showMemo = ref(false);
const showJsonEditor = ref(false);
const showTimestamp = ref(false);
const showAiChat = ref(false);
// 打开 AI 对话时带入的初始输入（来自搜索框）
const aiChatInitialText = ref("");
const showTranslator = ref(false);
// 打开翻译插件时带入的初始文本（来自搜索框）
const translatorInitialText = ref("");
const showCalc = ref(false);
const showTextTool = ref(false);
const textToolInitialText = ref("");
// 打开计算稿纸时带入的初始算式（来自搜索框）
const calcInitialExpression = ref("");
// 主搜索栏算式即时求值结果（非空时在推荐区顶部展示）
const calcResult = ref("");
const expandedRecent = ref(false);

// 窗口固定状态
const isPinned = ref(false);

// 状态持久化的 key
const STATE_STORAGE_KEY = "app_last_state";

// 数据统计
const clipboardCount = ref(0);
const memoCount = ref(0);

// 加载数据统计
async function loadDataStats() {
  try {
    // 加载剪贴板数量
    const clipboardItems = await getClipboardHistory();
    clipboardCount.value = clipboardItems.length;
    
    // 加载备忘录数量
    const memoItems = await getMemos();
    memoCount.value = memoItems.length;
  } catch (e) {
    console.error("Failed to load data stats:", e);
  }
}

// 保存当前状态
function saveCurrentState() {
  const state = {
    showSettings: showSettings.value,
    showPluginMarket: showPluginMarket.value,
    showClipboard: showClipboard.value,
    showMemo: showMemo.value,
    showJsonEditor: showJsonEditor.value,
    showTimestamp: showTimestamp.value,
    showAiChat: showAiChat.value,
    showTranslator: showTranslator.value,
    showCalc: showCalc.value,
    showTextTool: showTextTool.value,
    expandedRecent: expandedRecent.value,
    isPinned: isPinned.value,
  };
  localStorage.setItem(STATE_STORAGE_KEY, JSON.stringify(state));
}

// 恢复上次状态
function restoreLastState() {
  try {
    const savedState = localStorage.getItem(STATE_STORAGE_KEY);
    if (savedState) {
      const state = JSON.parse(savedState);
      showSettings.value = state.showSettings || false;
      showPluginMarket.value = state.showPluginMarket || false;
      showClipboard.value = state.showClipboard || false;
      showMemo.value = state.showMemo || false;
      showJsonEditor.value = state.showJsonEditor || false;
      showTimestamp.value = state.showTimestamp || false;
      showAiChat.value = state.showAiChat || false;
      showTranslator.value = state.showTranslator || false;
      showCalc.value = state.showCalc || false;
      showTextTool.value = state.showTextTool || false;
      expandedRecent.value = state.expandedRecent || false;
      isPinned.value = state.isPinned || false;
    }
  } catch (e) {
    console.error("Failed to restore last state:", e);
  }
}

// 切换固定状态
function togglePin() {
  isPinned.value = !isPinned.value;
  saveCurrentState();
}



// 加载插件列表
async function loadPlugins() {
  try {
    await initDB(); // 确保数据库已初始化
    const plugins = await getEnabledPlugins();
    allPlugins.value = plugins;
    console.log('Loaded plugins:', plugins.map(p => ({ id: p.id, name: p.name, keywords: p.keywords })));
  } catch (e) {
    console.error("Failed to load plugins:", e);
  }
}

// 定义所有插件及其关键词（已移除，改为从后端加载）

// 匹配插件
const matchedPlugins = computed(() => {
  const query = searchQuery.value.toLowerCase().trim();
  if (!query) return [];
  
  console.log('Searching for:', query, 'in plugins:', allPlugins.value.length);
  
  // 使用评分系统进行更精确的匹配
  const scored = allPlugins.value
    .map(plugin => {
      let score = 0;
      let matched = false;
      
      for (const keyword of plugin.keywords) {
        const keywordLower = keyword.toLowerCase();
        
        // 完全匹配 - 最高分
        if (keywordLower === query) {
          score = Math.max(score, 100);
          matched = true;
        }
        // 关键词以查询开头 - 高分
        else if (keywordLower.startsWith(query)) {
          score = Math.max(score, 80);
          matched = true;
        }
        // 查询以关键词开头 - 中等分
        else if (query.startsWith(keywordLower)) {
          score = Math.max(score, 60);
          matched = true;
        }
        // 关键词包含查询 - 低分
        else if (keywordLower.includes(query)) {
          score = Math.max(score, 40);
          matched = true;
        }
      }
      
      return { plugin, score, matched };
    })
    .filter(item => item.matched)
    .sort((a, b) => b.score - a.score)
    .map(item => item.plugin);
  
  return scored;
});

// 是否显示搜索结果
const showSearchResults = computed(() => searchQuery.value.length > 0 && (apps.value.length > 0 || matchedPlugins.value.length > 0));

// 是否显示智能推荐（什么都没匹配到时）
const showSmartRecommendations = computed(() => {
  if (searchQuery.value.length === 0) return false;
  if (apps.value.length > 0 || matchedPlugins.value.length > 0) return false;

  // 算式即时计算结果优先展示
  if (calcResult.value) return true;

  // 检测是否需要翻译（包含中文或英文）
  const hasChinese = /[\u4e00-\u9fa5]/.test(searchQuery.value);
  const hasEnglish = /[a-zA-Z]/.test(searchQuery.value);

  return hasChinese || hasEnglish;
});

// 智能推荐的插件
const smartRecommendedPlugins = computed(() => {
  const query = searchQuery.value;
  const hasChinese = /[\u4e00-\u9fa5]/.test(query);
  const hasEnglish = /[a-zA-Z]/.test(query);

  const recommendations = [];

  // 算式有效时，计算稿纸为第一推荐（Enter 直接进入并带入算式）
  if (calcResult.value) {
    const calcPlugin = allPlugins.value.find(p => p.id === 'calc');
    if (calcPlugin) {
      recommendations.push(calcPlugin);
    }
  }

  // 如果包含中文或英文，推荐翻译
  if (hasChinese || hasEnglish) {
    const translatorPlugin = allPlugins.value.find(p => p.id === 'translator');
    if (translatorPlugin) {
      recommendations.push(translatorPlugin);
    }
  }

  // 总是推荐AI对话
  const aiPlugin = allPlugins.value.find(p => p.id === 'ai');
  if (aiPlugin) {
    recommendations.push(aiPlugin);
  }

  return recommendations;
});

// 判断输入是否可能是算式（含数字且含运算符/括号，或形如函数调用）
function looksLikeMath(query: string): boolean {
  if (!/\d/.test(query)) return false;
  if (/[+\-*/%^()]/.test(query)) return true;
  return /(sqrt|sin|cos|tan|log|ln|abs|round|floor|ceil|exp|min|max|pow|cbrt|log2|asin|acos|atan)\s*\(/i.test(query);
}

// 主搜索栏算式即时求值（防抖）
let calcTimer: number | null = null;

async function checkCalcExpression(query: string) {
  if (!looksLikeMath(query)) {
    calcResult.value = "";
    return;
  }
  try {
    const result = await invoke<string>("evaluate_expression", { expr: query });
    // 结果与输入相同（纯数字回显）时不展示
    calcResult.value = result === query.trim() ? "" : result;
  } catch {
    calcResult.value = "";
  }
}

// 鼠标悬停指令行时同步键盘选中（uTools 式）
function onCommandHover(idx: number) {
  if (selectedIndex.value !== idx) selectedIndex.value = idx;
}

// 选中行滚动跟随（指令行 + 应用行）
watch(selectedIndex, () => {
  nextTick(() => {
    document
      .querySelector(".command-row.selected, .result-item.selected")
      ?.scrollIntoView({ block: "nearest" });
  });
});

// 防抖搜索
let searchTimer: number | null = null;

async function searchApps(query: string) {
  try {
    if (query.trim()) {
      apps.value = await invoke<App[]>("get_apps", { query });
      selectedIndex.value = 0;
    } else {
      apps.value = [];
    }
  } catch (e) {
    console.error("Failed to search apps:", e);
    apps.value = [];
  }
}

// 加载最近使用的应用
async function loadRecentApps() {
  try {
    const allApps = await invoke<App[]>("get_apps", { query: null });
    recentApps.value = allApps;
  } catch (e) {
    console.error("Failed to load recent apps:", e);
  }
}

// 显示的最近应用（根据展开状态）
const displayedRecentApps = computed(() => {
  return expandedRecent.value ? recentApps.value : recentApps.value.slice(0, 8);
});

// 切换展开状态
function toggleExpandRecent() {
  expandedRecent.value = !expandedRecent.value;
  saveCurrentState(); // 保存状态
}

watch(searchQuery, (newQuery) => {
  // 自动去除首尾空格
  const trimmedQuery = newQuery.trim();
  if (trimmedQuery !== newQuery) {
    searchQuery.value = trimmedQuery;
    return; // 等待下一次触发
  }
  
  if (searchTimer) {
    clearTimeout(searchTimer);
  }
  searchTimer = setTimeout(() => {
    searchApps(trimmedQuery);
  }, 150);

  // 算式即时求值（与搜索并行，仅在无匹配时展示）
  if (calcTimer) {
    clearTimeout(calcTimer);
  }
  if (!trimmedQuery) {
    calcResult.value = "";
  } else {
    calcTimer = setTimeout(() => {
      calcTimer = null;
      checkCalcExpression(trimmedQuery);
    }, 250);
  }
});

async function handleSelect(app: App) {
  try {
    await invoke("launch_app", { path: app.path });
    searchQuery.value = "";
    apps.value = [];
  } catch (e) {
    console.error("Failed to launch app:", e);
    alert(`启动应用失败: ${e}`);
  }
}

function handleKeydown(e: KeyboardEvent) {
  // 如果在设置界面、插件市场或插件中，ESC 返回主界面
  if (showSettings.value || showPluginMarket.value || showClipboard.value || showMemo.value || showJsonEditor.value || showTimestamp.value || showAiChat.value || showTranslator.value || showCalc.value || showTextTool.value) {
    if (e.key === "Escape") {
      showSettings.value = false;
      showPluginMarket.value = false;
      showClipboard.value = false;
      showMemo.value = false;
      showJsonEditor.value = false;
      showTimestamp.value = false;
      showAiChat.value = false;
      showTranslator.value = false;
      showCalc.value = false;
      saveCurrentState(); // 保存当前状态（主页面）
      nextTick(() => focusInput());
    }
    return;
  }

  // 如果有搜索结果（应用或插件）
  if (showSearchResults.value) {
    const totalResults = matchedPlugins.value.length + apps.value.length;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex.value = Math.min(selectedIndex.value + 1, totalResults - 1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
    } else if (e.key === "Enter") {
      e.preventDefault();
      // 插件指令优先：选中的是插件
      if (selectedIndex.value < matchedPlugins.value.length) {
        selectPlugin(matchedPlugins.value[selectedIndex.value]);
      }
      // 选中的是应用
      else {
        const appIndex = selectedIndex.value - matchedPlugins.value.length;
        if (appIndex < apps.value.length) {
          handleSelect(apps.value[appIndex]);
        }
      }
    }
  }
  // 如果显示智能推荐
  else if (showSmartRecommendations.value) {
    if (e.key === "Enter" && smartRecommendedPlugins.value.length > 0) {
      e.preventDefault();
      // 打开第一个推荐的插件
      selectPlugin(smartRecommendedPlugins.value[0]);
    }
  }
  
  if (e.key === "Escape") {
    if (searchQuery.value) {
      searchQuery.value = "";
      apps.value = [];
    } else {
      invoke("hide_window");
    }
  }
}

function openSettings() {
  showSettings.value = true;
  saveCurrentState(); // 保存状态
}

function closeSettings() {
  showSettings.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  nextTick(() => focusInput());
}

function openPluginMarket() {
  showSettings.value = false;
  showPluginMarket.value = true;
  saveCurrentState(); // 保存状态
}

function closePluginMarket() {
  showPluginMarket.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  nextTick(() => focusInput());
}

function openClipboard() {
  showClipboard.value = true;
  searchQuery.value = "";
  apps.value = [];
  saveCurrentState(); // 保存状态
}

function closeClipboard() {
  showClipboard.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  loadDataStats(); // 刷新数据统计
  nextTick(() => focusInput());
}

function openMemo() {
  showMemo.value = true;
  searchQuery.value = "";
  apps.value = [];
  saveCurrentState(); // 保存状态
}

function closeMemo() {
  showMemo.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  loadDataStats(); // 刷新数据统计
  nextTick(() => focusInput());
}

function openJsonEditor() {
  showJsonEditor.value = true;
  searchQuery.value = "";
  apps.value = [];
  saveCurrentState(); // 保存状态
}

function closeJsonEditor() {
  showJsonEditor.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  nextTick(() => focusInput());
}

function openTimestamp() {
  showTimestamp.value = true;
  searchQuery.value = "";
  apps.value = [];
  saveCurrentState(); // 保存状态
}

function closeTimestamp() {
  showTimestamp.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  nextTick(() => focusInput());
}

// 从搜索框抓取待处理文本：仅当内容不是通过插件关键词匹配进来时
// （关键词匹配时 query 是"翻译/fy"这类指令词，不应作为内容带入）
function grabInitialText(pluginId: string): string {
  const fromKeyword = matchedPlugins.value.some(p => p.id === pluginId);
  return fromKeyword ? "" : searchQuery.value;
}

function openAiChat() {
  aiChatInitialText.value = grabInitialText('ai');
  showAiChat.value = true;
  searchQuery.value = "";
  apps.value = [];
  saveCurrentState(); // 保存状态
}

function closeAiChat() {
  showAiChat.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  nextTick(() => focusInput());
}

function openTranslator() {
  // 搜索框内容作为初始文本带入（uTools 式：输入即翻译）
  translatorInitialText.value = grabInitialText('translator');
  showTranslator.value = true;
  searchQuery.value = "";
  apps.value = [];
  saveCurrentState(); // 保存状态
}

function closeTranslator() {
  showTranslator.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  nextTick(() => focusInput());
}

function openCalc() {
  // 搜索框算式带入计算稿纸
  calcInitialExpression.value = grabInitialText('calc');
  showCalc.value = true;
  searchQuery.value = "";
  apps.value = [];
  saveCurrentState(); // 保存状态
}

function closeCalc() {
  showCalc.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  nextTick(() => focusInput());
}

function openTextTool() {
  // 搜索框内容作为初始输入带入
  textToolInitialText.value = grabInitialText('text');
  showTextTool.value = true;
  searchQuery.value = "";
  apps.value = [];
  saveCurrentState(); // 保存状态
}

function closeTextTool() {
  showTextTool.value = false;
  saveCurrentState(); // 保存当前状态（主页面）
  nextTick(() => focusInput());
}

// 选择插件
function selectPlugin(plugin: Plugin) {
  // 根据插件 ID 执行对应操作
  switch (plugin.id) {
    case 'clipboard':
      openClipboard();
      break;
    case 'memo':
      openMemo();
      break;
    case 'json':
      openJsonEditor();
      break;
    case 'timestamp':
      openTimestamp();
      break;
    case 'ai':
      openAiChat();
      break;
    case 'translator':
      openTranslator();
      break;
    case 'calc':
      openCalc();
      break;
    case 'text':
      openTextTool();
      break;
    default:
      alert(`${plugin.name} 开发中...`);
  }
}

function focusInput() {
  if (searchInput.value) {
    searchInput.value.focus();
  }
}

let unlistenClearSearch: (() => void) | null = null;
let unlistenFocus: (() => void) | null = null;
let unlistenOpenSettings: (() => void) | null = null;

onMounted(async () => {
  window.addEventListener("keydown", handleKeydown);
  
  // 检查 URL 参数，如果有 plugin 参数则直接打开对应插件
  const urlParams = new URLSearchParams(window.location.search);
  const pluginParam = urlParams.get('plugin');
  
  if (pluginParam) {
    // 根据插件 ID 打开对应插件
    switch (pluginParam) {
      case 'json':
        showJsonEditor.value = true;
        break;
      case 'clipboard':
        showClipboard.value = true;
        break;
      case 'memo':
        showMemo.value = true;
        break;
      case 'timestamp':
        showTimestamp.value = true;
        break;
      case 'ai':
        showAiChat.value = true;
        break;
      case 'translator':
        showTranslator.value = true;
        break;
      case 'calc':
        showCalc.value = true;
        break;
      case 'text':
        showTextTool.value = true;
        break;
    }
    return; // 不执行后续的初始化逻辑
  }
  
  // 加载插件列表
  await loadPlugins();
  
  // 加载最近使用的应用
  await loadRecentApps();
  
  // 加载数据统计
  await loadDataStats();

  // 监听来自后端的清空搜索事件
  unlistenClearSearch = await listen<void>("clear-search", () => {
    searchQuery.value = "";
    apps.value = [];
    showSettings.value = false;
    showPluginMarket.value = false;
    showClipboard.value = false;
    showMemo.value = false;
    showJsonEditor.value = false;
    showTimestamp.value = false;
    showAiChat.value = false;
    showTranslator.value = false;
    showCalc.value = false;
    showTextTool.value = false;
    saveCurrentState(); // 保存当前状态（主页面）
    nextTick(() => focusInput());
  });

  // 监听打开设置事件（来自托盘菜单）
  unlistenOpenSettings = await listen<void>("open-settings", () => {
    openSettings();
  });

  // 监听窗口失去焦点事件（添加延迟，避免刚显示就隐藏）
  let lastShowTime = 0;
  const currentWindow = getCurrentWindow();
  
  // 记录窗口显示时间并恢复状态
  const recordShowTime = () => {
    lastShowTime = Date.now();
    // 窗口显示时恢复上次的状态
    restoreLastState();
  };
  
  // 监听窗口显示事件
  currentWindow.listen('tauri://focus', recordShowTime);
  
  unlistenFocus = await currentWindow.onFocusChanged(({ payload: focused }) => {
    if (!focused) {
      // 如果窗口被固定，不自动隐藏
      if (isPinned.value) {
        return;
      }
      
      // 只有在窗口显示超过 300ms 后才允许自动隐藏
      const timeSinceShow = Date.now() - lastShowTime;
      if (timeSinceShow > 300) {
        saveCurrentState(); // 失去焦点前保存状态（不清除）
        invoke("hide_window");
      }
    } else {
      // 窗口获得焦点时记录时间并恢复状态
      recordShowTime();
    }
  });

  // 初始聚焦
  nextTick(() => focusInput());
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  if (unlistenClearSearch) {
    unlistenClearSearch();
  }
  if (unlistenOpenSettings) {
    unlistenOpenSettings();
  }
  if (unlistenFocus) {
    unlistenFocus();
  }
});
</script>

<template>
  <div class="app-container">
    <!-- 时间戳转换插件 -->
    <TimestampPlugin v-if="showTimestamp" @close="closeTimestamp" />

    <!-- AI 对话插件 -->
    <AiChatPlugin v-else-if="showAiChat" :initial-input="aiChatInitialText" @close="closeAiChat" />

    <!-- 聚合翻译插件 -->
    <TranslatorPlugin v-else-if="showTranslator" :initial-text="translatorInitialText" @close="closeTranslator" />

    <!-- 计算稿纸插件 -->
    <CalcPlugin v-else-if="showCalc" :initial-expression="calcInitialExpression" @close="closeCalc" />
    <TextToolPlugin v-else-if="showTextTool" :initial-input="textToolInitialText" @close="closeTextTool" />

    <!-- JSON 编辑器插件 -->
    <JsonEditorPlugin v-else-if="showJsonEditor" @close="closeJsonEditor" />

    <!-- 剪贴板插件 -->
    <ClipboardPlugin v-else-if="showClipboard" @close="closeClipboard" />

    <!-- 备忘录插件 -->
    <MemoPlugin v-else-if="showMemo" @close="closeMemo" />

    <!-- 插件市场 -->
    <PluginMarket v-else-if="showPluginMarket" @close="closePluginMarket" />

    <!-- 设置界面 -->
    <Settings v-else-if="showSettings" @close="closeSettings" @open-market="openPluginMarket" />

    <!-- 主界面 -->
    <template v-else>
      <!-- 顶部搜索栏 -->
      <div class="search-header" data-tauri-drag-region>
        <div class="search-input-wrap">
          <svg class="search-glyph" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <circle cx="11" cy="11" r="7" />
            <path d="M21 21l-4.35-4.35" />
          </svg>
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            class="search-input"
            placeholder="搜索应用与插件，或输入算式、粘贴文本…"
            autocomplete="off"
            spellcheck="false"
          />
        </div>
        <button 
          class="pin-icon" 
          :class="{ pinned: isPinned }"
          title="固定窗口" 
          @click="togglePin"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 17v5"/>
            <path d="M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1z"/>
          </svg>
        </button>
        <button class="settings-icon" title="设置" @click="openSettings">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M21 4h-7" /><path d="M10 4H3" />
            <path d="M21 12h-9" /><path d="M8 12H3" />
            <path d="M21 20h-5" /><path d="M12 20H3" />
            <circle cx="12" cy="4" r="2" /><circle cx="10" cy="12" r="2" /><circle cx="14" cy="20" r="2" />
          </svg>
        </button>
      </div>

      <!-- 搜索结果 -->
      <div v-if="showSearchResults" class="search-results-container">
        <!-- 指令/插件结果置顶：输入关键词即命中，回车直达 -->
        <div v-if="matchedPlugins.length > 0" class="command-results">
          <div class="results-header">
            <h3>指令</h3>
          </div>
          <div
            v-for="(plugin, idx) in matchedPlugins"
            :key="plugin.id"
            class="command-row"
            :class="{ selected: idx === selectedIndex }"
            :style="{ '--gradient': plugin.gradient }"
            :data-cmd-idx="idx"
            @click="selectPlugin(plugin)"
            @mousemove="onCommandHover(idx)"
          >
            <div class="command-icon">{{ plugin.icon }}</div>
            <div class="command-body">
              <div class="command-name">{{ plugin.name }}</div>
              <div class="command-desc">{{ plugin.description }}</div>
            </div>
            <div v-if="idx === selectedIndex" class="command-enter">
              <kbd>↵</kbd>
            </div>
          </div>
        </div>

        <!-- 应用结果 -->
        <PluginList
          v-if="apps.length > 0"
          :apps="apps"
          :selected-index="selectedIndex - matchedPlugins.length"
          @select="handleSelect"
        />
      </div>

      <!-- 智能推荐（什么都没匹配到时） -->
      <div v-else-if="showSmartRecommendations" class="plugin-recommendations">
        <div class="recommendations-header">
          <h3>智能推荐</h3>
          <span class="hint-text">没找到？试试这些 ></span>
        </div>

        <div class="recommendations-content">
          <!-- 算式即时计算结果 -->
          <div v-if="calcResult" class="calc-inline" @click="openCalc">
            <div class="calc-inline-icon">🧮</div>
            <div class="calc-inline-body">
              <div class="calc-inline-expr">{{ searchQuery }}</div>
              <div class="calc-inline-hint">Enter 或点击打开计算稿纸</div>
            </div>
            <div class="calc-inline-result">= {{ calcResult }}</div>
          </div>
          <div class="recommendations-grid">
            <div
              v-for="plugin in smartRecommendedPlugins"
              :key="plugin.id"
              class="recommendation-item"
              :style="{ '--gradient': plugin.gradient }"
              @click="selectPlugin(plugin)"
            >
              <div class="plugin-icon-large">{{ plugin.icon }}</div>
              <div class="plugin-name-large">{{ plugin.name }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- 默认视图：最近使用 + 市场精选 -->
      <div v-else class="default-view">
        <!-- 最近使用 -->
        <div class="section">
          <div class="section-header">
            <h3>最近使用</h3>
            <span class="expand-hint" @click="toggleExpandRecent">
              {{ expandedRecent ? '收起' : '展开' }} ({{ recentApps.length }})
            </span>
          </div>
          <div class="recent-apps">
            <div
              v-for="app in displayedRecentApps"
              :key="app.id"
              class="recent-app-item"
              @click="handleSelect(app)"
            >
              <div class="app-icon-wrapper">
                <img v-if="app.icon" :src="app.icon" class="app-icon" alt="" />
                <div v-else class="app-icon-placeholder">📱</div>
              </div>
              <div class="app-name">{{ app.name }}</div>
            </div>
          </div>
        </div>

        <!-- 市场精选 -->
        <div class="section">
          <div class="section-header">
            <h3>快捷插件</h3>
          </div>
          <div class="market-grid">
            <div
              v-for="plugin in allPlugins.slice(0, 7)"
              :key="plugin.id"
              class="market-item"
              :style="{ '--gradient': plugin.gradient }"
              @click="selectPlugin(plugin)"
            >
              <div class="market-icon">{{ plugin.icon }}</div>
              <div class="market-info">
                <div class="market-name">{{ plugin.name }}</div>
                <div v-if="plugin.id === 'clipboard' && clipboardCount > 0" class="market-count">{{ clipboardCount }} 条记录</div>
                <div v-else-if="plugin.id === 'memo' && memoCount > 0" class="market-count">{{ memoCount }} 条备忘</div>
              </div>
            </div>
            <div class="market-item" style="--gradient: linear-gradient(135deg, #ff9a9e 0%, #fecfef 100%)" @click="openPluginMarket">
              <div class="market-icon">🛍️</div>
              <div class="market-name">更多插件...</div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.app-container {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  position: relative;
  background: rgba(24, 26, 33, 0.88);
  backdrop-filter: blur(40px) saturate(180%);
  -webkit-backdrop-filter: blur(40px) saturate(180%);
  border-radius: var(--r-window);
  overflow: hidden;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.45);
}

/* 顶部品牌微光，呼应图标蓝紫渐变 */
.app-container::before {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  background:
    radial-gradient(120% 50% at 18% -8%, rgba(10, 132, 255, 0.10), transparent 60%),
    radial-gradient(90% 40% at 88% -12%, rgba(125, 92, 255, 0.08), transparent 55%);
  z-index: 0;
}

.app-container > * {
  position: relative;
  z-index: 1;
}

/* 顶部搜索栏 - macOS 风格 */
.search-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 20px;
  background: rgba(18, 19, 24, 0.55);
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.search-input-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 10px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.04));
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: var(--r-ctl);
  padding: 0 14px;
  transition: border-color 180ms ease, box-shadow 180ms ease, background 180ms ease;
  -webkit-app-region: no-drag;
}

.search-input-wrap:focus-within {
  border-color: rgba(10, 132, 255, 0.65);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.05));
  box-shadow: 0 0 0 3px rgba(10, 132, 255, 0.18), 0 4px 18px rgba(10, 132, 255, 0.10);
}

.search-glyph {
  color: rgba(255, 255, 255, 0.4);
  flex-shrink: 0;
}

.search-input-wrap:focus-within .search-glyph {
  color: #0a84ff;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  padding: 11px 0;
  font-size: 16px;
  font-weight: 450;
  color: rgba(255, 255, 255, 0.95);
  outline: none;
  -webkit-app-region: no-drag;
}

.search-input::placeholder {
  color: rgba(255, 255, 255, 0.35);
  font-weight: 400;
}

.pin-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--r-ctl);
  color: rgba(255, 255, 255, 0.65);
  cursor: pointer;
  transition: all 150ms cubic-bezier(0.4, 0.0, 0.2, 1);
  -webkit-app-region: no-drag;
}

.pin-icon:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.95);
  transform: translateY(-1px);
}

.pin-icon.pinned {
  background: #0a84ff;
  border-color: #0a84ff;
  color: #fff;
  box-shadow: 0 2px 10px rgba(10, 132, 255, 0.4);
}

.pin-icon.pinned:hover {
  background: #0a6fdd;
  border-color: #0a6fdd;
}

.pin-icon:active {
  transform: translateY(0) scale(0.95);
}

.settings-icon {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--r-ctl);
  color: rgba(255, 255, 255, 0.65);
  cursor: pointer;
  transition: all 150ms cubic-bezier(0.4, 0.0, 0.2, 1);
  -webkit-app-region: no-drag;
}

.settings-icon:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.95);
  transform: translateY(-1px);
}

.settings-icon:active {
  transform: translateY(0) scale(0.95);
}

/* 默认视图 */
.default-view {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

/* 区块 */
.section {
  margin-bottom: 32px;
  animation: fadeIn 250ms cubic-bezier(0.4, 0.0, 0.2, 1);
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  padding: 0 4px;
}

.section-header h3 {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  letter-spacing: 1.5px;
  margin: 0;
}

.expand-hint {
  font-size: 12px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.45);
  cursor: pointer;
  transition: color 150ms cubic-bezier(0.4, 0.0, 0.2, 1);
}

.expand-hint:hover {
  color: #0a84ff;
}

/* 最近使用 - macOS 风格 */
.recent-apps {
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  gap: 12px;
}

.recent-app-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 12px 8px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.02));
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: var(--r-card);
  cursor: pointer;
  transition: all 180ms cubic-bezier(0.25, 0.1, 0.25, 1);
}

.recent-app-item:hover {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.09), rgba(255, 255, 255, 0.04));
  border-color: rgba(255, 255, 255, 0.16);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.recent-app-item:active {
  transform: translateY(0) scale(0.98);
}

.app-icon-wrapper {
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--r-card);
  overflow: hidden;
  background: rgba(255, 255, 255, 0.05);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.app-icon {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.app-icon-placeholder {
  font-size: 32px;
}

.app-name {
  font-size: 12px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.95);
  text-align: center;
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 市场精选 - macOS 风格 */
.market-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

.market-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 20px 16px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.025));
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--r-card);
  cursor: pointer;
  transition: all 200ms cubic-bezier(0.25, 0.1, 0.25, 1);
}

.market-item:hover {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.05));
  border-color: rgba(255, 255, 255, 0.18);
  transform: translateY(-3px);
  box-shadow: 0 12px 28px rgba(0, 0, 0, 0.35);
}

.market-item:active {
  transform: translateY(-1px) scale(0.98);
}

.market-icon {
  width: 52px;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 30px;
  border-radius: var(--r-card);
  background: var(--gradient, linear-gradient(135deg, #667eea 0%, #764ba2 100%));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.28),
    0 6px 14px rgba(0, 0, 0, 0.28);
}

.market-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 100%;
}

.market-name {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.95);
  text-align: center;
  line-height: 1.4;
}

.market-count {
  font-size: 11px;
  font-weight: 400;
  color: rgba(255, 255, 255, 0.65);
  text-align: center;
  padding: 2px 8px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: var(--r-ctl);
}

/* 搜索结果容器 */
.search-results-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
}

/* 指令行（插件关键词命中，置顶展示） */
.command-results {
  padding: 14px 16px 6px;
}

.command-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 14px;
  margin-bottom: 4px;
  border-radius: var(--r-ctl);
  border: 1px solid transparent;
  cursor: pointer;
  transition: background 140ms ease, border-color 140ms ease;
  animation: rowIn 180ms cubic-bezier(0.25, 0.1, 0.25, 1) backwards;
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

.command-row.selected {
  background: linear-gradient(90deg, rgba(10, 132, 255, 0.2), rgba(125, 92, 255, 0.12));
  border-color: rgba(10, 132, 255, 0.35);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);
}

.command-row:active {
  transform: scale(0.995);
}

.command-icon {
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  border-radius: var(--r-card);
  background: var(--gradient, linear-gradient(135deg, #667eea 0%, #764ba2 100%));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.28),
    0 4px 12px rgba(0, 0, 0, 0.25);
}

.command-body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.command-name {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.92);
}

.command-desc {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.42);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.command-enter {
  flex-shrink: 0;
}

.command-enter kbd {
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

/* 插件推荐 - macOS 风格 */
.plugin-recommendations {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: transparent;
  overflow: hidden;
}

.recommendations-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  background: rgba(18, 19, 24, 0.55);
  border-bottom: 1px solid rgba(255, 255, 255, 0.07);
}

.recommendations-header h3 {
  font-size: 12px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  letter-spacing: 1.5px;
  margin: 0;
}

.hint-text {
  font-size: 12px;
  font-weight: 500;
  color: #0a84ff;
  cursor: pointer;
  transition: color 150ms cubic-bezier(0.4, 0.0, 0.2, 1);
}

.hint-text:hover {
  color: #0a6fdd;
}

.recommendations-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.recommendations-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
  gap: 16px;
  max-width: 100%;
}

.recommendation-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 20px 16px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.025));
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: var(--r-card);
  cursor: pointer;
  transition: all 200ms cubic-bezier(0.25, 0.1, 0.25, 1);
}

.recommendation-item:hover {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.05));
  border-color: rgba(255, 255, 255, 0.18);
  transform: translateY(-4px);
  box-shadow: 0 12px 28px rgba(0, 0, 0, 0.35);
}

.recommendation-item:active {
  transform: translateY(-2px) scale(0.98);
}

.plugin-icon-large {
  width: 64px;
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 36px;
  border-radius: var(--r-card);
  background: var(--gradient, linear-gradient(135deg, #667eea 0%, #764ba2 100%));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.28),
    0 8px 18px rgba(0, 0, 0, 0.3);
}

.plugin-name-large {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.95);
  text-align: center;
  line-height: 1.4;
}

/* 滚动条 - macOS 风格 */
.default-view::-webkit-scrollbar,
.recommendations-content::-webkit-scrollbar {
  width: 8px;
}

/* 算式即时计算卡片 */
.calc-inline {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 18px;
  margin-bottom: 16px;
  background: linear-gradient(135deg, rgba(48, 207, 208, 0.12), rgba(51, 8, 103, 0.2));
  border: 1px solid rgba(48, 207, 208, 0.25);
  border-radius: var(--r-card);
  cursor: pointer;
  transition: all 150ms cubic-bezier(0.4, 0.0, 0.2, 1);
}

.calc-inline:hover {
  border-color: rgba(48, 207, 208, 0.5);
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(48, 207, 208, 0.15);
}

.calc-inline-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  border-radius: var(--r-ctl);
  background: linear-gradient(135deg, #30cfd0 0%, #330867 100%);
  flex-shrink: 0;
}

.calc-inline-body {
  flex: 1;
  min-width: 0;
}

.calc-inline-expr {
  font-family: "SF Mono", Menlo, Consolas, monospace;
  font-size: 13px;
  color: rgba(255, 255, 255, 0.9);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.calc-inline-hint {
  margin-top: 3px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
}

.calc-inline-result {
  font-family: "SF Mono", Menlo, Consolas, monospace;
  font-size: 20px;
  font-weight: 600;
  color: #4fe0c8;
  white-space: nowrap;
  flex-shrink: 0;
}

.default-view::-webkit-scrollbar-track,
.recommendations-content::-webkit-scrollbar-track {
  background: transparent;
}

.default-view::-webkit-scrollbar-thumb,
.recommendations-content::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
  border: 2px solid transparent;
  background-clip: padding-box;
}

.default-view::-webkit-scrollbar-thumb:hover,
.recommendations-content::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.25);
  background-clip: padding-box;
}
</style>
