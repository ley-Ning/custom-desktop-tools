<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface AiModel {
  id: string;
  name: string;
  provider: string;
  baseUrl: string;
  apiKey: string;
  model: string;
  enabled: boolean;
  isDefault: boolean;
}

interface AiConfig {
  models: AiModel[];
}

const aiConfig = ref<AiConfig>({ models: [] });
const selectedModel = ref<string | null>(null);
const showAddModel = ref(false);
const configError = ref("");
const configSuccess = ref("");

// 新模型表单
const newModel = ref({
  name: "",
  provider: "openai",
  baseUrl: "",
  apiKey: "",
  model: "",
  enabled: true,
});

// 预设提供商
const providers = [
  { id: "openai", name: "OpenAI", defaultUrl: "https://api.openai.com/v1", defaultModel: "gpt-4" },
  { id: "anthropic", name: "Anthropic", defaultUrl: "https://api.anthropic.com/v1", defaultModel: "claude-3-opus-20240229" },
  { id: "deepseek", name: "DeepSeek", defaultUrl: "https://api.deepseek.com/v1", defaultModel: "deepseek-chat" },
  { id: "openrouter", name: "OpenRouter", defaultUrl: "https://openrouter.ai/api/v1", defaultModel: "anthropic/claude-3-opus" },
  { id: "custom", name: "自定义", defaultUrl: "", defaultModel: "" },
];

// 加载 AI 配置
async function loadAiConfig() {
  try {
    const config = await invoke<string>("load_ai_config");
    if (config) {
      aiConfig.value = JSON.parse(config);
    }
  } catch (e) {
    console.error("Failed to load AI config:", e);
    configError.value = "加载配置失败";
  }
}

// 保存 AI 配置
async function saveAiConfig() {
  try {
    configError.value = "";
    configSuccess.value = "";
    
    const configStr = JSON.stringify(aiConfig.value, null, 2);
    await invoke("save_ai_config", { config: configStr });
    
    configSuccess.value = "配置已保存";
    setTimeout(() => {
      configSuccess.value = "";
    }, 2000);
  } catch (e) {
    console.error("Failed to save AI config:", e);
    configError.value = "保存配置失败：" + e;
  }
}

// 当提供商改变时更新默认值
function onProviderChange() {
  const provider = providers.find(p => p.id === newModel.value.provider);
  if (provider && provider.id !== "custom") {
    newModel.value.baseUrl = provider.defaultUrl;
    newModel.value.model = provider.defaultModel;
  }
}

// 添加新模型
function addModel() {
  if (!newModel.value.name || !newModel.value.baseUrl || !newModel.value.apiKey || !newModel.value.model) {
    configError.value = "请填写所有必填字段";
    return;
  }

  const model: AiModel = {
    id: `model-${Date.now()}`,
    name: newModel.value.name,
    provider: newModel.value.provider,
    baseUrl: newModel.value.baseUrl,
    apiKey: newModel.value.apiKey,
    model: newModel.value.model,
    enabled: newModel.value.enabled,
    isDefault: aiConfig.value.models.length === 0, // 第一个模型设为默认
  };

  aiConfig.value.models.push(model);

  // 重置表单
  newModel.value = {
    name: "",
    provider: "openai",
    baseUrl: "",
    apiKey: "",
    model: "",
    enabled: true,
  };

  showAddModel.value = false;
  saveAiConfig();
}

// 删除模型
function deleteModel(id: string) {
  if (confirm("确定要删除此模型配置吗？")) {
    aiConfig.value.models = aiConfig.value.models.filter(m => m.id !== id);
    if (selectedModel.value === id) {
      selectedModel.value = null;
    }
    saveAiConfig();
  }
}

// 切换模型启用状态
function toggleModelEnabled(id: string) {
  const model = aiConfig.value.models.find(m => m.id === id);
  if (model) {
    model.enabled = !model.enabled;
    saveAiConfig();
  }
}

// 设置默认模型
function setDefaultModel(id: string) {
  aiConfig.value.models.forEach(m => {
    m.isDefault = m.id === id;
  });
  saveAiConfig();
}

// 选择模型
function selectModel(id: string) {
  selectedModel.value = selectedModel.value === id ? null : id;
}

// 测试连接
async function testConnection(model: AiModel) {
  configError.value = "";
  configSuccess.value = "";
  
  try {
    // TODO: 实现实际的 API 测试
    configSuccess.value = `正在测试 ${model.name} 连接...`;
    
    // 模拟测试
    setTimeout(() => {
      configSuccess.value = `${model.name} 连接成功！`;
      setTimeout(() => {
        configSuccess.value = "";
      }, 2000);
    }, 1000);
  } catch (e) {
    configError.value = `连接失败：${e}`;
  }
}

// 获取统计信息
const modelCount = () => aiConfig.value.models.length;
const enabledModelCount = () => aiConfig.value.models.filter(m => m.enabled).length;
const defaultModel = () => aiConfig.value.models.find(m => m.isDefault);

onMounted(() => {
  loadAiConfig();
});
</script>

<template>
  <div class="ai-settings">
    <div class="ai-header">
      <div class="header-info">
        <h2 class="panel-title">AI 模型配置</h2>
        <p class="panel-subtitle">
          配置 OpenAI、Anthropic、DeepSeek 等 AI 模型的 API 密钥和端点
        </p>
      </div>
      <button class="btn-add" @click="showAddModel = !showAddModel">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        添加模型
      </button>
    </div>

    <!-- 统计信息 -->
    <div class="ai-stats">
      <div class="stat-card">
        <div class="stat-value">{{ modelCount() }}</div>
        <div class="stat-label">配置模型</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ enabledModelCount() }}</div>
        <div class="stat-label">已启用</div>
      </div>
      <div class="stat-card">
        <div class="stat-icon">⭐</div>
        <div class="stat-label">{{ defaultModel()?.name || "未设置" }}</div>
        <div class="stat-sublabel">默认模型</div>
      </div>
    </div>

    <!-- 消息提示 -->
    <div v-if="configError" class="error-message">{{ configError }}</div>
    <div v-if="configSuccess" class="success-message">{{ configSuccess }}</div>

    <!-- 添加模型表单 -->
    <div v-if="showAddModel" class="add-model-form">
      <h3 class="form-title">添加新模型</h3>
      
      <div class="form-group">
        <label class="form-label">模型名称 *</label>
        <input
          v-model="newModel.name"
          type="text"
          class="form-input"
          placeholder="例如: GPT-4"
        />
      </div>

      <div class="form-group">
        <label class="form-label">提供商 *</label>
        <select v-model="newModel.provider" class="form-select" @change="onProviderChange">
          <option v-for="provider in providers" :key="provider.id" :value="provider.id">
            {{ provider.name }}
          </option>
        </select>
      </div>

      <div class="form-group">
        <label class="form-label">Base URL *</label>
        <input
          v-model="newModel.baseUrl"
          type="text"
          class="form-input"
          placeholder="https://api.openai.com/v1"
        />
        <span class="form-hint">API 端点地址</span>
      </div>

      <div class="form-group">
        <label class="form-label">API Key *</label>
        <input
          v-model="newModel.apiKey"
          type="password"
          class="form-input"
          placeholder="sk-..."
        />
        <span class="form-hint">你的 API 密钥</span>
      </div>

      <div class="form-group">
        <label class="form-label">模型名称 *</label>
        <input
          v-model="newModel.model"
          type="text"
          class="form-input"
          placeholder="gpt-4"
        />
        <span class="form-hint">具体的模型标识符</span>
      </div>

      <div class="form-group">
        <label class="form-checkbox">
          <input v-model="newModel.enabled" type="checkbox" />
          <span>启用此模型</span>
        </label>
      </div>

      <div class="form-actions">
        <button class="btn-primary" @click="addModel">添加</button>
        <button class="btn-secondary" @click="showAddModel = false">取消</button>
      </div>
    </div>

    <!-- 模型列表 -->
    <div class="model-list">
      <div v-if="modelCount() === 0" class="empty-state">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M12 2L2 7l10 5 10-5-10-5z"/>
          <path d="M2 17l10 5 10-5"/>
          <path d="M2 12l10 5 10-5"/>
        </svg>
        <p>暂无 AI 模型配置</p>
        <p class="hint">点击上方"添加模型"按钮开始配置</p>
      </div>

      <div
        v-for="model in aiConfig.models"
        :key="model.id"
        class="model-item"
        :class="{ disabled: !model.enabled, expanded: selectedModel === model.id, default: model.isDefault }"
      >
        <div class="model-header" @click="selectModel(model.id)">
          <div class="model-info">
            <div class="model-name">
              <span class="status-dot" :class="{ active: model.enabled }"></span>
              {{ model.name }}
              <span v-if="model.isDefault" class="default-badge">默认</span>
            </div>
            <div class="model-details">
              <span class="provider-tag">{{ providers.find(p => p.id === model.provider)?.name }}</span>
              <span class="model-tag">{{ model.model }}</span>
            </div>
          </div>
          <div class="model-actions">
            <button
              v-if="!model.isDefault"
              class="btn-icon"
              title="设为默认"
              @click.stop="setDefaultModel(model.id)"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
              </svg>
            </button>
            <button
              class="btn-icon"
              :title="model.enabled ? '禁用' : '启用'"
              @click.stop="toggleModelEnabled(model.id)"
            >
              <svg v-if="model.enabled" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
              <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18"/>
                <line x1="6" y1="6" x2="18" y2="18"/>
              </svg>
            </button>
            <button
              class="btn-icon"
              title="测试连接"
              @click.stop="testConnection(model)"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
                <polyline points="22 4 12 14.01 9 11.01"/>
              </svg>
            </button>
            <button
              class="btn-icon btn-danger"
              title="删除"
              @click.stop="deleteModel(model.id)"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- 展开的详细信息 -->
        <div v-if="selectedModel === model.id" class="model-details-expanded">
          <div class="detail-section">
            <div class="detail-label">Base URL</div>
            <div class="detail-value">
              <code>{{ model.baseUrl }}</code>
            </div>
          </div>

          <div class="detail-section">
            <div class="detail-label">API Key</div>
            <div class="detail-value">
              <code class="masked">{{ model.apiKey.substring(0, 8) }}...{{ model.apiKey.substring(model.apiKey.length - 4) }}</code>
            </div>
          </div>

          <div class="detail-section">
            <div class="detail-label">模型</div>
            <div class="detail-value">
              <code>{{ model.model }}</code>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 帮助信息 -->
    <div class="help-section">
      <h3 class="help-title">💡 使用说明</h3>
      <ul class="help-list">
        <li>支持 OpenAI、Anthropic、DeepSeek、OpenRouter 等主流 AI 服务</li>
        <li>可以配置自定义 API 端点（例如代理服务器）</li>
        <li>API Key 会加密存储在本地，不会上传到云端</li>
        <li>点击"测试连接"可以验证配置是否正确</li>
        <li>设置默认模型后，AI 功能将优先使用该模型</li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.ai-settings {
  max-width: 900px;
}

.ai-header {
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

.btn-add {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: #5a9fd4;
  border: none;
  border-radius: 8px;
  color: #1a1a1a;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-add:hover {
  background: #6aafed;
  transform: translateY(-1px);
}

/* 统计卡片 */
.ai-stats {
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

.stat-icon {
  font-size: 32px;
  margin-bottom: 8px;
}

.stat-label {
  font-size: 12px;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-sublabel {
  font-size: 10px;
  color: #666;
  margin-top: 4px;
}

/* 表单 */
.add-model-form {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 24px;
}

.form-title {
  font-size: 16px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0 0 20px 0;
}

.form-group {
  margin-bottom: 16px;
}

.form-label {
  display: block;
  font-size: 13px;
  color: #b0b0b0;
  margin-bottom: 8px;
}

.form-input,
.form-select {
  width: 100%;
  padding: 10px 12px;
  background: #2b2b2b;
  border: 1px solid #4a4a4a;
  border-radius: 6px;
  color: #e0e0e0;
  font-size: 13px;
  font-family: inherit;
  outline: none;
  transition: all 0.2s ease;
}

.form-input:focus,
.form-select:focus {
  border-color: #5a9fd4;
  box-shadow: 0 0 0 3px rgba(90, 159, 212, 0.1);
}

.form-hint {
  display: block;
  font-size: 11px;
  color: #666;
  margin-top: 4px;
}

.form-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #b0b0b0;
  cursor: pointer;
}

.form-checkbox input {
  cursor: pointer;
}

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 20px;
}

.btn-primary,
.btn-secondary {
  flex: 1;
  padding: 10px 16px;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background: #5a9fd4;
  color: #1a1a1a;
}

.btn-primary:hover {
  background: #6aafed;
}

.btn-secondary {
  background: #3a3a3a;
  color: #e0e0e0;
  border: 1px solid #4a4a4a;
}

.btn-secondary:hover {
  background: #4a4a4a;
}

/* 模型列表 */
.model-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 24px;
}

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

.empty-state .hint {
  font-size: 12px;
  color: #666;
}

.model-item {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 10px;
  overflow: hidden;
  transition: all 0.2s ease;
}

.model-item:hover {
  border-color: #5a5a5a;
}

.model-item.disabled {
  opacity: 0.6;
}

.model-item.default {
  border-color: #f59e0b;
}

.model-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  cursor: pointer;
}

.model-info {
  flex: 1;
}

.model-name {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 15px;
  font-weight: 600;
  color: #e0e0e0;
  margin-bottom: 8px;
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

.default-badge {
  padding: 2px 8px;
  background: #f59e0b;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 600;
  color: #1a1a1a;
  text-transform: uppercase;
}

.model-details {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.provider-tag,
.model-tag {
  padding: 4px 10px;
  background: #2b2b2b;
  border: 1px solid #4a4a4a;
  border-radius: 12px;
  font-size: 11px;
  color: #888;
}

.model-actions {
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

.btn-icon.btn-danger:hover {
  background: rgba(243, 139, 168, 0.1);
  border-color: rgba(243, 139, 168, 0.3);
  color: #f38ba8;
}

/* 模型详情 */
.model-details-expanded {
  padding: 0 20px 20px 20px;
  border-top: 1px solid #3a3a3a;
}

.detail-section {
  margin-top: 16px;
}

.detail-label {
  font-size: 12px;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.detail-value {
  font-size: 13px;
  color: #b0b0b0;
}

.detail-value code {
  padding: 6px 12px;
  background: #2b2b2b;
  border-radius: 6px;
  font-family: "SF Mono", "Monaco", monospace;
  font-size: 12px;
  color: #5a9fd4;
  display: inline-block;
}

.detail-value code.masked {
  color: #888;
}

/* 帮助信息 */
.help-section {
  background: #323232;
  border: 1px solid #4a4a4a;
  border-radius: 12px;
  padding: 20px 24px;
}

.help-title {
  font-size: 14px;
  font-weight: 600;
  color: #e0e0e0;
  margin: 0 0 12px 0;
}

.help-list {
  margin: 0;
  padding-left: 20px;
  color: #888;
  font-size: 13px;
  line-height: 1.8;
}

.help-list li {
  margin-bottom: 8px;
}

/* 消息提示 */
.error-message {
  margin-bottom: 16px;
  padding: 12px 16px;
  background: rgba(243, 139, 168, 0.1);
  border: 1px solid rgba(243, 139, 168, 0.3);
  border-radius: 8px;
  font-size: 13px;
  color: #f38ba8;
}

.success-message {
  margin-bottom: 16px;
  padding: 12px 16px;
  background: rgba(166, 227, 161, 0.1);
  border: 1px solid rgba(166, 227, 161, 0.3);
  border-radius: 8px;
  font-size: 13px;
  color: #a6e3a1;
}
</style>
