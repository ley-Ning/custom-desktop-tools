# 开发者指南

## 🛠️ 开发环境设置

### 必需工具

1. **Node.js** (v18+)
2. **pnpm** (v8+)
3. **Rust** (latest stable)
4. **Xcode Command Line Tools** (macOS)

### 安装步骤

```bash
# 1. 克隆项目
git clone <repository-url>
cd my-utools

# 2. 安装依赖
pnpm install

# 3. 启动开发服务器
pnpm tauri dev
```

## 📁 项目结构

```
my-utools/
├── src/                        # Vue 前端代码
│   ├── components/            # Vue 组件
│   │   ├── App.vue           # 主应用
│   │   ├── PluginList.vue    # 搜索结果
│   │   ├── Settings.vue      # 设置界面
│   │   ├── PluginMarket.vue  # 插件市场
│   │   ├── ClipboardPlugin.vue # 剪贴板插件
│   │   └── MemoPlugin.vue    # 备忘录插件
│   ├── styles/               # 样式文件
│   ├── types/                # TypeScript 类型
│   └── main.ts               # 入口文件
├── src-tauri/                 # Rust 后端代码
│   ├── src/
│   │   ├── lib.rs            # 核心逻辑
│   │   └── main.rs           # 入口文件
│   ├── Cargo.toml            # Rust 依赖
│   └── tauri.conf.json       # Tauri 配置
├── docs/                      # 文档
└── package.json              # Node 依赖
```

## 🔧 开发工作流

### 1. 启动开发服务器

```bash
pnpm tauri dev
```

这会：
- 启动 Vite 开发服务器（前端）
- 编译并运行 Rust 代码（后端）
- 打开应用窗口
- 启用热重载

### 2. 前端开发

#### 创建新组件

```vue
<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);
</script>

<template>
  <div>{{ count }}</div>
</template>

<style scoped>
/* 组件样式 */
</style>
```

#### 调用后端命令

```typescript
import { invoke } from "@tauri-apps/api/core";

// 调用 Rust 命令
const result = await invoke<string>("command_name", {
  param1: "value1",
  param2: 123,
});
```

### 3. 后端开发

#### 添加新命令

```rust
#[command]
async fn my_command(param: String) -> Result<String, String> {
    // 实现逻辑
    Ok(format!("Result: {}", param))
}

// 在 run() 函数中注册
.invoke_handler(tauri::generate_handler![
    my_command,
    // ... 其他命令
])
```

#### 数据持久化

```rust
use tauri_plugin_store::StoreExt;

#[command]
async fn save_data(app: AppHandle, data: String) -> Result<(), String> {
    if let Ok(store) = app.store("data.json") {
        let _ = store.set("key", serde_json::json!(data));
        let _ = store.save();
    }
    Ok(())
}
```

## 🧩 添加新插件

### 1. 创建插件组件

```bash
# 创建新文件
touch src/components/MyPlugin.vue
```

### 2. 实现插件 UI

```vue
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{
  close: [];
}>();

// 插件逻辑
</script>

<template>
  <div class="my-plugin">
    <!-- 插件 UI -->
  </div>
</template>

<style scoped>
/* 插件样式 */
</style>
```

### 3. 添加后端命令

```rust
#[command]
async fn my_plugin_command() -> Result<String, String> {
    // 实现逻辑
    Ok("Success".to_string())
}
```

### 4. 集成到主应用

```vue
<!-- App.vue -->
<script setup lang="ts">
import MyPlugin from "./components/MyPlugin.vue";

const showMyPlugin = ref(false);

function openMyPlugin() {
  showMyPlugin.value = true;
}
</script>

<template>
  <MyPlugin v-if="showMyPlugin" @close="showMyPlugin = false" />
</template>
```

### 5. 添加快捷入口

```vue
<!-- 在主界面添加卡片 -->
<div class="market-item" @click="openMyPlugin">
  <div class="market-icon">🎯</div>
  <div class="market-name">我的插件</div>
</div>
```

## 🎨 样式指南

### 颜色变量

```css
/* 主要颜色 */
--bg-primary: #2b2b2b;      /* 主背景 */
--bg-secondary: #323232;    /* 次要背景 */
--bg-hover: #3a3a3a;        /* 悬停背景 */
--border: #4a4a4a;          /* 边框 */
--text-primary: #e0e0e0;    /* 主文字 */
--text-secondary: #888;     /* 次要文字 */
--accent: #5a9fd4;          /* 强调色 */
```

### 组件样式模板

```vue
<style scoped>
.my-component {
  background: #2b2b2b;
  border: 1px solid #4a4a4a;
  border-radius: 8px;
  padding: 16px;
  color: #e0e0e0;
  transition: all 0.2s ease;
}

.my-component:hover {
  background: #3a3a3a;
  border-color: #5a5a5a;
}
</style>
```

## 🧪 测试

### 手动测试

```bash
# 启动应用
pnpm tauri dev

# 测试功能
1. 按 Alt+Space 唤出窗口
2. 测试搜索功能
3. 测试插件功能
4. 测试设置功能
```

### 调试技巧

#### 前端调试

```typescript
// 使用 console.log
console.log("Debug:", data);

// 使用浏览器开发者工具
// 在应用中按 Cmd+Option+I (macOS)
```

#### 后端调试

```rust
// 使用 println!
println!("Debug: {:?}", data);

// 使用 dbg! 宏
dbg!(&variable);

// 查看终端输出
```

## 📦 构建和发布

### 开发构建

```bash
pnpm tauri build --debug
```

### 生产构建

```bash
pnpm tauri build
```

构建产物位置：
- macOS: `src-tauri/target/release/bundle/macos/`
- DMG: `src-tauri/target/release/bundle/dmg/`

## 🐛 常见问题

### Q: 编译错误

**A:** 确保 Rust 和 Node.js 版本正确：
```bash
rustc --version  # 应该是最新稳定版
node --version   # 应该是 v18+
```

### Q: 热重载不工作

**A:** 重启开发服务器：
```bash
# 停止当前服务器 (Ctrl+C)
pnpm tauri dev
```

### Q: 窗口无法显示

**A:** 检查 tauri.conf.json 中的窗口配置：
```json
{
  "app": {
    "windows": [{
      "visible": true,
      "center": true
    }]
  }
}
```

### Q: 命令调用失败

**A:** 确保命令已注册：
```rust
.invoke_handler(tauri::generate_handler![
    your_command_name,
])
```

## 📝 代码规范

### TypeScript

```typescript
// 使用 const 和 let，避免 var
const name = "value";
let count = 0;

// 使用类型注解
function greet(name: string): string {
  return `Hello, ${name}`;
}

// 使用接口定义类型
interface User {
  id: string;
  name: string;
}
```

### Rust

```rust
// 使用 snake_case 命名
fn my_function() {}

// 使用 Result 处理错误
fn do_something() -> Result<String, String> {
    Ok("Success".to_string())
}

// 使用 async/await
#[command]
async fn async_command() -> Result<(), String> {
    // 异步操作
    Ok(())
}
```

### Vue

```vue
<script setup lang="ts">
// 使用 Composition API
import { ref, computed } from "vue";

// 使用 ref 和 reactive
const count = ref(0);

// 使用 computed
const doubled = computed(() => count.value * 2);
</script>

<template>
  <!-- 使用语义化标签 -->
  <div class="container">
    <h1>{{ title }}</h1>
  </div>
</template>

<style scoped>
/* 使用 scoped 样式 */
</style>
```

## 🔍 性能优化

### 前端优化

1. **使用 computed 缓存计算结果**
```typescript
const filteredItems = computed(() => {
  return items.value.filter(item => item.active);
});
```

2. **使用 v-show 代替 v-if（频繁切换）**
```vue
<div v-show="isVisible">Content</div>
```

3. **使用防抖和节流**
```typescript
let timer: number | null = null;
function debounce(fn: Function, delay: number) {
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => fn(), delay) as unknown as number;
}
```

### 后端优化

1. **使用缓存**
```rust
use std::sync::Mutex;

static CACHE: Mutex<Option<Vec<String>>> = Mutex::new(None);
```

2. **避免阻塞操作**
```rust
use tokio::task;

#[command]
async fn heavy_task() -> Result<String, String> {
    task::spawn_blocking(|| {
        // 耗时操作
    }).await.map_err(|e| e.to_string())
}
```

## 📚 学习资源

### Tauri
- [官方文档](https://tauri.app/v1/guides/)
- [API 参考](https://tauri.app/v1/api/js/)

### Vue 3
- [官方文档](https://vuejs.org/)
- [Composition API](https://vuejs.org/guide/extras/composition-api-faq.html)

### Rust
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📧 联系方式

- GitHub: [@lewen](https://github.com/lewen)
- Email: your.email@example.com

---

Happy Coding! 🚀
