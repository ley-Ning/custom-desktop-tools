# MCP (Model Context Protocol) 配置指南

## 功能概述

My uTools 现在支持 MCP (Model Context Protocol) 配置，允许你连接和管理各种 AI 模型上下文协议服务器。

## 如何使用

### 1. 打开 MCP 配置

1. 启动应用（Alt+Space）
2. 点击右上角设置按钮
3. 在左侧菜单选择"MCP 配置"

### 2. 添加 MCP 服务器

点击"添加服务器"按钮，填写以下信息：

- **服务器名称**：唯一标识符（例如：weather-api）
- **命令**：启动服务器的命令（例如：uvx、node、python）
- **参数**：命令行参数，每行一个
- **环境变量**：KEY=VALUE 格式，每行一个
- **自动批准工具**：不需要用户确认的工具名称，逗号分隔

### 3. 管理服务器

- **启用/禁用**：点击服务器右侧的开关图标
- **查看详情**：点击服务器卡片展开详细信息
- **删除服务器**：点击删除图标（垃圾桶）

## 配置示例

### 示例 1：天气服务器

```json
{
  "name": "weather-api",
  "command": "uvx",
  "args": ["mcp-server-weather@latest"],
  "env": {
    "API_KEY": "your_api_key_here",
    "LOG_LEVEL": "info"
  },
  "disabled": false,
  "autoApprove": ["get_weather", "get_forecast"]
}
```

### 示例 2：文件系统服务器

```json
{
  "name": "filesystem",
  "command": "uvx",
  "args": ["mcp-server-filesystem@latest", "--root=/Users/username/Documents"],
  "disabled": false,
  "autoApprove": ["read_file", "list_directory"]
}
```

### 示例 3：数据库服务器

```json
{
  "name": "postgres",
  "command": "node",
  "args": ["./mcp-servers/postgres/index.js"],
  "env": {
    "DATABASE_URL": "postgresql://localhost:5432/mydb",
    "DB_USER": "admin",
    "DB_PASSWORD": "password"
  },
  "disabled": false,
  "autoApprove": []
}
```

## 配置文件位置

MCP 配置保存在：
```
~/.local/share/com.lewen.my-utools/mcp.json
```

你可以手动编辑此文件，但建议通过 UI 界面进行配置。

## 常见 MCP 服务器

### 使用 uvx 安装（推荐）

```bash
# 安装 uv（Python 包管理器）
curl -LsSf https://astral.sh/uv/install.sh | sh

# 运行 MCP 服务器（无需安装）
uvx mcp-server-weather@latest
uvx mcp-server-filesystem@latest
uvx mcp-server-git@latest
```

### 流行的 MCP 服务器

1. **天气服务器**
   - 包名：`mcp-server-weather`
   - 功能：获取天气信息和预报

2. **文件系统服务器**
   - 包名：`mcp-server-filesystem`
   - 功能：读写文件、列出目录

3. **Git 服务器**
   - 包名：`mcp-server-git`
   - 功能：Git 操作、查看提交历史

4. **数据库服务器**
   - 包名：`mcp-server-postgres`、`mcp-server-mysql`
   - 功能：数据库查询和操作

5. **浏览器自动化**
   - 包名：`mcp-server-puppeteer`
   - 功能：网页抓取、自动化测试

## 安全注意事项

1. **环境变量**：不要在配置中存储敏感信息（API 密钥、密码）
2. **自动批准**：谨慎设置自动批准的工具，确保它们是安全的
3. **服务器来源**：只使用可信来源的 MCP 服务器
4. **权限控制**：定期检查服务器的权限和访问范围

## 故障排除

### 服务器无法启动

1. 检查命令是否正确安装（例如：`which uvx`）
2. 验证参数格式是否正确
3. 查看环境变量是否设置正确
4. 检查日志输出

### 配置不生效

1. 确保配置已保存（查看成功提示）
2. 重启应用使配置生效
3. 检查 JSON 格式是否正确

### 权限问题

1. 确保应用有读写配置文件的权限
2. 检查配置目录是否存在：`~/.local/share/com.lewen.my-utools/`

## 开发自定义 MCP 服务器

如果你想开发自己的 MCP 服务器，请参考：

- [MCP 官方文档](https://modelcontextprotocol.io/)
- [MCP SDK](https://github.com/modelcontextprotocol/sdk)
- [示例服务器](https://github.com/modelcontextprotocol/servers)

## 更新日志

- **v0.1.0**：初始 MCP 配置支持
  - 添加/删除/启用/禁用服务器
  - 配置环境变量和自动批准工具
  - 统计信息显示

## 反馈和支持

如有问题或建议，请提交 Issue 或 Pull Request。
