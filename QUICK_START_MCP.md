# MCP 配置快速启动指南

## 🚀 5 分钟快速上手

### 第 1 步：启动应用

```bash
pnpm tauri dev
```

### 第 2 步：打开 MCP 配置

1. 按 `Alt+Space` 唤起应用
2. 点击右上角 **设置** 按钮（三层图标）
3. 在左侧菜单点击 **"MCP 配置"**

### 第 3 步：添加第一个服务器

点击右上角 **"+ 添加服务器"** 按钮，填写：

```
服务器名称：weather-api
命令：uvx
参数：
  mcp-server-weather@latest
环境变量：
  API_KEY=demo_key
  LOG_LEVEL=info
自动批准工具：get_weather, get_forecast
```

点击 **"添加"** 按钮。

### 第 4 步：查看配置

✅ 你应该看到：
- 统计信息显示：总数 1，已启用 1
- 服务器卡片显示绿色状态点
- 服务器名称和命令显示正确

### 第 5 步：测试功能

#### 查看详情
点击服务器卡片，展开查看详细配置

#### 禁用服务器
点击右侧的 ✓ 图标，状态点变灰

#### 启用服务器
再次点击图标，状态点变绿

#### 删除服务器
点击 🗑️ 图标，确认删除

## 📖 完整文档

- **使用指南**：[MCP_SETUP.md](./MCP_SETUP.md)
- **功能总结**：[MCP_FEATURE_SUMMARY.md](./MCP_FEATURE_SUMMARY.md)
- **测试指南**：[TEST_MCP.md](./TEST_MCP.md)
- **实现总结**：[MCP_IMPLEMENTATION.md](./MCP_IMPLEMENTATION.md)

## 🎯 常见问题

### Q: 配置保存在哪里？
A: `~/.local/share/com.lewen.my-utools/mcp.json`

### Q: 如何安装 uvx？
A: 
```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

### Q: 配置后需要重启吗？
A: 是的，修改配置后需要重启应用才能生效

### Q: 如何备份配置？
A: 复制配置文件：
```bash
cp ~/.local/share/com.lewen.my-utools/mcp.json ~/mcp-backup.json
```

## 💡 提示

- 使用 `uvx` 命令可以直接运行 MCP 服务器，无需安装
- 环境变量支持多行，每行一个 KEY=VALUE
- 自动批准的工具将不需要用户确认即可执行
- 点击服务器卡片可以展开/折叠详细信息

## 🎉 完成！

现在你已经成功配置了第一个 MCP 服务器！

继续探索更多功能，或查看完整文档了解高级用法。
