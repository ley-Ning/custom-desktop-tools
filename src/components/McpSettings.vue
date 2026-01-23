<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface McpServer {
  name: string;
  command: string;
  args: string[];
  env?: Record<string, string>;
  disabled: boolean;
  autoApprove: string[];
}

interface McpConfig {
  mcpServers: Record<string, McpServer>;
}

const mcpConfig = ref<McpConfig>({ mcpServers: {} });
const selectedServer = ref<string | null>(null);
const showAddServer = ref(false);
const showPasteJson = ref(false);
const pasteJsonText = ref("");
const configError = ref("");
const configSuccess = ref("");

// 新服务器表单
const newServer = ref({
  name: "",
  command: "",
  args: "",
  env: "",
  disabled: false,
  autoApprove: "",
});

// 加载 MCP 配置
async function loadMcpConfig() {
  try {
    const config = await invoke<string>("load_mcp_config");
    if (config) {
      mcpConfig.value = JSON.parse(config);
    }
  } catch (e) {
    console.error("Failed to load MCP config:", e);
    configError.value = "加载配置失败";
  }
}

// 保存 MCP 配置
async function saveMcpConfig() {
  try {
    configError.value = "";
    configSuccess.value = "";
    
    const configStr = JSON.stringify(mcpConfig.value, null, 2);
    await invoke("save_mcp_config", { config: configStr });
    
    configSuccess.value = "配置已保存";
    setTimeout(() => {
      configSuccess.value = "";
    }, 2000);
  } catch (e) {
    console.error("Failed to save MCP config:", e);
    configError.value = "保存配置失败：" + e;
  }
}

// 添加新服务器
function addServer() {
  if (!newServer.value.name || !newServer.value.command) {
    configError.value = "服务器名称和命令不能为空";
    return;
  }

  const args = newServer.value.args
    .split("\n")
    .map(s => s.trim())
    .filter(s => s.length > 0);

  const env: Record<string, string> = {};
  if (newServer.value.env) {
    newServer.value.env.split("\n").forEach(line => {
      const [key, ...valueParts] = line.split("=");
      if (key && valueParts.length > 0) {
        env[key.trim()] = valueParts.join("=").trim();
      }
    });
  }

  const autoApprove = newServer.value.autoApprove
    .split(",")
    .map(s => s.trim())
    .filter(s => s.length > 0);

  mcpConfig.value.mcpServers[newServer.value.name] = {
    name: newServer.value.name,
    command: newServer.value.command,
    args,
    env: Object.keys(env).length > 0 ? env : undefined,
    disabled: newServer.value.disabled,
    autoApprove,
  };

  // 重置表单
  newServer.value = {
    name: "",
    command: "",
    args: "",
    env: "",
    disabled: false,
    autoApprove: "",
  };

  showAddServer.value = false;
  saveMcpConfig();
}

// 从 JSON 导入配置
function importFromJson() {
  try {
    configError.value = "";
    
    if (!pasteJsonText.value.trim()) {
      configError.value = "请粘贴 JSON 配置";
      return;
    }

    const parsed = JSON.parse(pasteJsonText.value);
    
    // 验证格式
    if (!parsed.mcpServers || typeof parsed.mcpServers !== 'object') {
      configError.value = "无效的配置格式，需要包含 mcpServers 对象";
      return;
    }

    // 合并配置（不覆盖现有的）
    let addedCount = 0;
    let skippedCount = 0;
    
    for (const [name, server] of Object.entries(parsed.mcpServers)) {
      if (mcpConfig.value.mcpServers[name]) {
        skippedCount++;
      } else {
        mcpConfig.value.mcpServers[name] = server as McpServer;
        addedCount++;
      }
    }

    pasteJsonText.value = "";
    showPasteJson.value = false;
    
    if (addedCount > 0) {
      saveMcpConfig();
      configSuccess.value = `成功导入 ${addedCount} 个服务器${skippedCount > 0 ? `，跳过 ${skippedCount} 个已存在的服务器` : ''}`;
      setTimeout(() => {
        configSuccess.value = "";
      }, 3000);
    } else {
      configError.value = "没有新服务器被导入（所有服务器已存在）";
    }
  } catch (e) {
    configError.value = "JSON 格式错误：" + (e as Error).message;
  }
}

// 删除服务器
function deleteServer(name: string) {
  if (confirm(`确定要删除服务器 "${name}" 吗？`)) {
    delete mcpConfig.value.mcpServers[name];
    if (selectedServer.value === name) {
      selectedServer.value = null;
    }
    saveMcpConfig();
  }
}

// 切换服务器启用状态
function toggleServerEnabled(name: string) {
  const server = mcpConfig.value.mcpServers[name];
  if (server) {
    server.disabled = !server.disabled;
    saveMcpConfig();
  }
}

// 选择服务器
function selectServer(name: string) {
  selectedServer.value = selectedServer.value === name ? null : name;
}

// 获取服务器数量
const serverCount = () => Object.keys(mcpConfig.value.mcpServers).length;
const enabledServerCount = () => 
  Object.values(mcpConfig.value.mcpServers).filter(s => !s.disabled).length;

onMounted(() => {
  loadMcpConfig();
});
</script>

<template>
  <div class="mcp-settings">
    <div class="mcp-header">
      <div class="header-info">
        <h2 class="panel-title">MCP 服务器配置</h2>
        <p class="panel-subtitle">
          Model Context Protocol - 配置和管理 AI 模型上下文协议服务器
        </p>
      </div>
      <div class="header-actions">
        <button class="btn-add btn-secondary-action" @click="showPasteJson = !showPasteJson">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>
            <rect x="8" y="2" width="8" height="4" rx="1" ry="1"/>
          </svg>
          粘贴 JSON
        </button>
        <button class="btn-add" @click="showAddServer = !showAddServer">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"/>
            <line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          添加服务器
        </button>
      </div>
    </div>

    <!-- 统计信息 -->
    <div class="mcp-stats">
      <div class="stat-card">
        <div class="stat-value">{{ serverCount() }}</div>
        <div class="stat-label">总服务器数</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ enabledServerCount() }}</div>
        <div class="stat-label">已启用</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ serverCount() - enabledServerCount() }}</div>
        <div class="stat-label">已禁用</div>
      </div>
    </div>

    <!-- 消息提示 -->
    <div v-if="configError" class="error-message">{{ configError }}</div>
    <div v-if="configSuccess" class="success-message">{{ configSuccess }}</div>

    <!-- 粘贴 JSON 表单 -->
    <div v-if="showPasteJson" class="add-server-form">
      <h3 class="form-title">从 JSON 导入配置</h3>
      <p class="form-hint">
        粘贴完整的 MCP 配置 JSON，例如：
      </p>
      <pre class="json-example">{
  "mcpServers": {
    "yxbj-mcp": {
      "command": "npx",
      "args": ["yxbj-mcp"],
      "env": {
        "YINXIANG_AUTH_TOKEN": "your_token_here"
      }
    }
  }
}</pre>
      
      <div class="form-group">
        <label class="form-label">JSON 配置</label>
        <textarea
          v-model="pasteJsonText"
          class="form-textarea json-textarea"
          rows="12"
          placeholder="粘贴 JSON 配置..."
        ></textarea>
      </div>

      <div class="form-actions">
        <button class="btn-primary" @click="importFromJson">导入</button>
        <button class="btn-secondary" @click="showPasteJson = false; pasteJsonText = ''">取消</button>
      </div>
    </div>

    <!-- 添加服务器表单 -->
    <div v-if="showAddServer" class="add-server-form">
      <h3 class="form-title">添加新服务器</h3>
      
      <div class="form-group">
        <label class="form-label">服务器名称 *</label>
        <input
          v-model="newServer.name"
          type="text"
          class="form-input"
          placeholder="例如: weather-api"
        />
      </div>

      <div class="form-group">
        <label class="form-label">命令 *</label>
        <input
          v-model="newServer.command"
          type="text"
          class="form-input"
          placeholder="例如: uvx 或 node"
        />
      </div>

      <div class="form-group">
        <label class="form-label">参数（每行一个）</label>
        <textarea
          v-model="newServer.args"
          class="form-textarea"
          rows="3"
          placeholder="例如:&#10;mcp-server-weather@latest&#10;--port=3000"
        ></textarea>
      </div>

      <div class="form-group">
        <label class="form-label">环境变量（KEY=VALUE 格式，每行一个）</label>
        <textarea
          v-model="newServer.env"
          class="form-textarea"
          rows="2"
          placeholder="例如:&#10;API_KEY=your_key&#10;LOG_LEVEL=info"
        ></textarea>
      </div>

      <div class="form-group">
        <label class="form-label">自动批准的工具（逗号分隔）</label>
        <input
          v-model="newServer.autoApprove"
          type="text"
          class="form-input"
          placeholder="例如: get_weather, get_forecast"
        />
      </div>

      <div class="form-group">
        <label class="form-checkbox">
          <input v-model="newServer.disabled" type="checkbox" />
          <span>创建后禁用此服务器</span>
        </label>
      </div>

      <div class="form-actions">
        <button class="btn-primary" @click="addServer">添加</button>
        <button class="btn-secondary" @click="showAddServer = false">取消</button>
      </div>
    </div>

    <!-- 服务器列表 -->
    <div class="server-list">
      <div v-if="serverCount() === 0" class="empty-state">
        <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="2" y="3" width="20" height="14" rx="2"/>
          <line x1="8" y1="21" x2="16" y2="21"/>
          <line x1="12" y1="17" x2="12" y2="21"/>
        </svg>
        <p>暂无 MCP 服务器</p>
        <p class="hint">点击上方"添加服务器"按钮开始配置</p>
      </div>

      <div
        v-for="(server, name) in mcpConfig.mcpServers"
        :key="name"
        class="server-item"
        :class="{ disabled: server.disabled, expanded: selectedServer === name }"
      >
        <div class="server-header" @click="selectServer(name as string)">
          <div class="server-info">
            <div class="server-name">
              <span class="status-dot" :class="{ active: !server.disabled }"></span>
              {{ name }}
            </div>
            <div class="server-command">{{ server.command }} {{ server.args.join(" ") }}</div>
          </div>
          <div class="server-actions">
            <button
              class="btn-icon"
              :title="server.disabled ? '启用' : '禁用'"
              @click.stop="toggleServerEnabled(name as string)"
            >
              <svg v-if="server.disabled" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/>
              </svg>
              <svg v-else width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12"/>
              </svg>
            </button>
            <button
              class="btn-icon btn-danger"
              title="删除"
              @click.stop="deleteServer(name as string)"
            >
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
          </div>
        </div>

        <!-- 展开的详细信息 -->
        <div v-if="selectedServer === name" class="server-details">
          <div class="detail-section">
            <div class="detail-label">命令</div>
            <div class="detail-value">{{ server.command }}</div>
          </div>

          <div v-if="server.args.length > 0" class="detail-section">
            <div class="detail-label">参数</div>
            <div class="detail-value">
              <div v-for="(arg, idx) in server.args" :key="idx" class="detail-item">
                {{ arg }}
              </div>
            </div>
          </div>

          <div v-if="server.env && Object.keys(server.env).length > 0" class="detail-section">
            <div class="detail-label">环境变量</div>
            <div class="detail-value">
              <div v-for="(value, key) in server.env" :key="key" class="detail-item">
                <code>{{ key }}={{ value }}</code>
              </div>
            </div>
          </div>

          <div v-if="server.autoApprove.length > 0" class="detail-section">
            <div class="detail-label">自动批准工具</div>
            <div class="detail-value">
              <div class="tag-list">
                <span v-for="tool in server.autoApprove" :key="tool" class="tag">
                  {{ tool }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 帮助信息 -->
    <div class="help-section">
      <h3 class="help-title">💡 使用说明</h3>
      <ul class="help-list">
        <li>MCP 服务器需要单独安装，推荐使用 <code>uvx</code> 命令运行</li>
        <li>配置文件保存在：<code>~/.local/share/com.lewen.my-utools/mcp.json</code></li>
        <li>修改配置后需要重启应用才能生效</li>
        <li>自动批准的工具将不需要用户确认即可执行</li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.mcp-settings {
  max-width: 900px;
}

.mcp-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 24px;
}

.header-info {
  flex: 1;
}

.header-actions {
  display: flex;
  gap: 12px;
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

.btn-secondary-action {
  background: #3a3a3a;
  color: #e0e0e0;
  border: 1px solid #4a4a4a;
}

.btn-secondary-action:hover {
  background: #4a4a4a;
  border-color: #5a5a5a;
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

/* 表单 */
.add-server-form {
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
  margin: 0 0 12px 0;
}

.form-hint {
  font-size: 13px;
  color: #888;
  margin: 0 0 12px 0;
  line-height: 1.6;
}

.json-example {
  background: #2b2b2b;
  border: 1px solid #3a3a3a;
  border-radius: 6px;
  padding: 12px;
  font-family: "SF Mono", "Monaco", monospace;
  font-size: 12px;
  color: #5a9fd4;
  overflow-x: auto;
  margin-bottom: 16px;
}

.json-textarea {
  font-family: "SF Mono", "Monaco", monospace;
  font-size: 12px;
  line-height: 1.6;
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
.form-textarea {
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
.form-textarea:focus {
  border-color: #5a9fd4;
  box-shadow: 0 0 0 3px rgba(90, 159, 212, 0.1);
}

.form-textarea {
  resize: vertical;
  font-family: "SF Mono", "Monaco", monospace;
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

/* 服务器列表 */
.server-list {
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
  cursor: pointer;
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
  font-family: "SF Mono", "Monaco", monospace;
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

/* 服务器详情 */
.server-details {
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

.detail-item {
  padding: 6px 12px;
  background: #2b2b2b;
  border-radius: 6px;
  margin-bottom: 6px;
  font-family: "SF Mono", "Monaco", monospace;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tag {
  padding: 4px 10px;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  border-radius: 12px;
  font-size: 12px;
  color: #5a9fd4;
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

.help-list code {
  padding: 2px 6px;
  background: #2b2b2b;
  border-radius: 4px;
  font-family: "SF Mono", "Monaco", monospace;
  font-size: 12px;
  color: #5a9fd4;
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
