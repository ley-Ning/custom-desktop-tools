# AI 模型配置功能

## 🎉 新功能：AI 模型配置管理

### 功能概述

现在可以在设置中配置多个 AI 模型，支持自定义 Base URL 和 API Key，方便使用不同的 AI 服务提供商。

### 支持的 AI 服务

1. **OpenAI**
   - Base URL: `https://api.openai.com/v1`
   - 模型: gpt-4, gpt-3.5-turbo 等

2. **Anthropic (Claude)**
   - Base URL: `https://api.anthropic.com/v1`
   - 模型: claude-3-opus-20240229, claude-3-sonnet-20240229 等

3. **DeepSeek**
   - Base URL: `https://api.deepseek.com/v1`
   - 模型: deepseek-chat, deepseek-coder 等

4. **OpenRouter**
   - Base URL: `https://openrouter.ai/api/v1`
   - 模型: 支持多种模型路由

5. **自定义**
   - 支持任何兼容 OpenAI API 格式的服务
   - 可以配置代理服务器

## 如何使用

### 1. 打开 AI 模型配置

1. 启动应用（Alt+Space）
2. 点击右上角设置按钮
3. 在左侧菜单选择"AI 模型"

### 2. 添加 AI 模型

点击"+ 添加模型"按钮，填写以下信息：

#### 必填字段

- **模型名称**：给模型起个名字（例如：GPT-4）
- **提供商**：选择 AI 服务提供商
  - OpenAI
  - Anthropic
  - DeepSeek
  - OpenRouter
  - 自定义
- **Base URL**：API 端点地址
- **API Key**：你的 API 密钥
- **模型名称**：具体的模型标识符（例如：gpt-4）

#### 可选设置

- **启用此模型**：是否立即启用

### 3. 管理模型

#### 设置默认模型
- 点击星星图标 ⭐ 设置为默认模型
- 默认模型会在 AI 功能中优先使用

#### 启用/禁用模型
- 点击 ✓ 或 ✗ 图标切换状态
- 禁用的模型不会在 AI 功能中显示

#### 测试连接
- 点击测试图标验证配置是否正确
- 会尝试连接 API 并返回结果

#### 查看详情
- 点击模型卡片展开详细信息
- 显示 Base URL、API Key（部分隐藏）、模型名称

#### 删除模型
- 点击删除图标（垃圾桶）
- 确认后删除配置

## 配置示例

### 示例 1：OpenAI GPT-4

```json
{
  "name": "GPT-4",
  "provider": "openai",
  "baseUrl": "https://api.openai.com/v1",
  "apiKey": "sk-proj-...",
  "model": "gpt-4",
  "enabled": true,
  "isDefault": true
}
```

### 示例 2：Claude 3 Opus

```json
{
  "name": "Claude 3 Opus",
  "provider": "anthropic",
  "baseUrl": "https://api.anthropic.com/v1",
  "apiKey": "sk-ant-...",
  "model": "claude-3-opus-20240229",
  "enabled": true,
  "isDefault": false
}
```

### 示例 3：DeepSeek

```json
{
  "name": "DeepSeek Chat",
  "provider": "deepseek",
  "baseUrl": "https://api.deepseek.com/v1",
  "apiKey": "sk-...",
  "model": "deepseek-chat",
  "enabled": true,
  "isDefault": false
}
```

### 示例 4：自定义代理

```json
{
  "name": "自定义 GPT-4",
  "provider": "custom",
  "baseUrl": "https://your-proxy.com/v1",
  "apiKey": "your-key",
  "model": "gpt-4",
  "enabled": true,
  "isDefault": false
}
```

## 配置文件位置

AI 模型配置保存在：
```
~/.local/share/com.lewen.my-utools/ai-config.json
```

## 安全说明

### API Key 安全

1. **本地存储**：API Key 存储在本地配置文件中
2. **不上传云端**：所有配置仅保存在本地
3. **部分显示**：界面中只显示 API Key 的前 8 位和后 4 位
4. **建议**：
   - 定期更换 API Key
   - 不要分享配置文件
   - 使用有限额度的 API Key

### 权限控制

- 确保配置文件只有当前用户可读写
- 不要在公共场所展示配置界面

## 使用场景

### 1. 多模型切换

配置多个 AI 模型，根据不同任务选择最合适的模型：
- GPT-4：复杂推理任务
- Claude：长文本处理
- DeepSeek：代码生成

### 2. 成本优化

- 日常任务使用便宜的模型
- 重要任务使用高级模型
- 设置不同的默认模型

### 3. 代理服务

- 配置代理服务器绕过网络限制
- 使用第三方 API 服务
- 自建 API 网关

### 4. 开发测试

- 配置测试环境的 API
- 使用模拟 API 进行开发
- 快速切换不同环境

## 故障排除

### 问题：连接失败

**可能原因：**
1. API Key 错误
2. Base URL 不正确
3. 网络问题
4. API 配额用完

**解决方案：**
1. 检查 API Key 是否正确
2. 验证 Base URL 格式
3. 测试网络连接
4. 查看 API 服务商控制台

### 问题：配置不保存

**解决方案：**
1. 检查文件权限
2. 确保配置目录存在
3. 查看控制台错误信息

### 问题：模型不显示

**解决方案：**
1. 确认模型已启用
2. 检查配置文件格式
3. 重启应用

## API 格式说明

### OpenAI 兼容格式

大多数 AI 服务都兼容 OpenAI 的 API 格式：

```bash
POST {baseUrl}/chat/completions
Content-Type: application/json
Authorization: Bearer {apiKey}

{
  "model": "gpt-4",
  "messages": [
    {"role": "user", "content": "Hello"}
  ]
}
```

### 特殊格式

某些服务可能需要特殊的请求格式，请参考各服务商的文档。

## 获取 API Key

### OpenAI
1. 访问 https://platform.openai.com/
2. 注册/登录账号
3. 进入 API Keys 页面
4. 创建新的 API Key

### Anthropic
1. 访问 https://console.anthropic.com/
2. 注册/登录账号
3. 进入 API Keys 页面
4. 创建新的 API Key

### DeepSeek
1. 访问 https://platform.deepseek.com/
2. 注册/登录账号
3. 进入 API Keys 页面
4. 创建新的 API Key

### OpenRouter
1. 访问 https://openrouter.ai/
2. 注册/登录账号
3. 进入 Keys 页面
4. 创建新的 API Key

## 最佳实践

1. **命名规范**
   - 使用清晰的模型名称
   - 包含版本信息
   - 标注用途

2. **安全管理**
   - 定期更换 API Key
   - 使用环境变量（未来支持）
   - 限制 API Key 权限

3. **成本控制**
   - 设置使用限额
   - 监控 API 调用次数
   - 选择合适的模型

4. **性能优化**
   - 缓存常用响应
   - 批量处理请求
   - 选择就近的服务器

## 未来计划

- [ ] 支持环境变量配置
- [ ] API 使用统计
- [ ] 成本追踪
- [ ] 模型性能对比
- [ ] 批量导入/导出
- [ ] 配置模板
- [ ] 自动测试连接
- [ ] 响应时间监控

## 相关文档

- [MCP_SETUP.md](./MCP_SETUP.md) - MCP 配置指南
- [FEATURES.md](./FEATURES.md) - 功能列表
- [PROJECT_STATUS.md](./PROJECT_STATUS.md) - 项目状态

---

**版本**：v0.1.0  
**更新时间**：2026-01-23  
**作者**：Kiro AI Assistant
