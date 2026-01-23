# macOS 设计规范应用

## ✅ 已实现的 macOS 设计特性

### 🎨 视觉效果

1. **毛玻璃效果（Vibrancy）**
   - ✅ 窗口背景使用 `backdrop-filter: blur(40px) saturate(180%)`
   - ✅ 半透明背景 `rgba(30, 30, 30, 0.85)`
   - ✅ 真实的毛玻璃质感

2. **圆角设计**
   - ✅ 窗口圆角：18px
   - ✅ 卡片圆角：12-14px
   - ✅ 按钮圆角：8-10px
   - ✅ 输入框圆角：10px

3. **阴影系统**
   - ✅ 小阴影：`0 2px 8px rgba(0, 0, 0, 0.15)`
   - ✅ 中阴影：`0 4px 16px rgba(0, 0, 0, 0.25)`
   - ✅ 大阴影：`0 8px 32px rgba(0, 0, 0, 0.35)`

4. **颜色系统**
   - ✅ 主背景：`rgba(30, 30, 30, 0.85)`
   - ✅ 次背景：`rgba(40, 40, 40, 0.75)`
   - ✅ 强调色：`#007AFF`（macOS 蓝色）
   - ✅ 文字颜色：
     - 主要：`rgba(255, 255, 255, 0.95)`
     - 次要：`rgba(255, 255, 255, 0.65)`
     - 三级：`rgba(255, 255, 255, 0.45)`

### 🎭 动画效果

1. **缓动函数**
   - ✅ 使用 `cubic-bezier(0.4, 0.0, 0.2, 1)` - macOS 标准缓动
   - ✅ 快速动画：150ms
   - ✅ 正常动画：250ms
   - ✅ 慢速动画：350ms

2. **交互动画**
   - ✅ 悬停：`translateY(-2px)` + 阴影增强
   - ✅ 点击：`scale(0.98)` 缩放反馈
   - ✅ 淡入：`fadeIn` 动画
   - ✅ 缩放进入：`scaleIn` 动画

### 📝 字体系统

1. **SF Pro 字体**
   - ✅ 使用 `-apple-system` 系统字体
   - ✅ 标题：15-20px，font-weight: 600
   - ✅ 正文：13-14px，font-weight: 400-500
   - ✅ 说明：11-12px，font-weight: 400
   - ✅ 字间距：-0.2px 到 -0.3px

### 🖱️ 交互设计

1. **按钮**
   - ✅ 半透明背景 + 边框
   - ✅ 悬停时背景变亮 + 上移
   - ✅ 点击时缩放反馈
   - ✅ 主要按钮使用蓝色

2. **输入框**
   - ✅ 半透明背景
   - ✅ 聚焦时蓝色边框 + 光晕
   - ✅ 圆角 10px
   - ✅ 平滑过渡

3. **卡片**
   - ✅ 半透明背景 + 细边框
   - ✅ 悬停时上移 + 阴影增强
   - ✅ 圆角 12-14px
   - ✅ 点击缩放反馈

4. **滚动条**
   - ✅ macOS 原生样式
   - ✅ 半透明滚动条
   - ✅ 悬停时变亮
   - ✅ 圆角 4px

## 📐 设计规范

### 间距系统
```css
--macos-spacing-xs: 4px;   /* 超小间距 */
--macos-spacing-sm: 8px;   /* 小间距 */
--macos-spacing-md: 12px;  /* 中间距 */
--macos-spacing-lg: 16px;  /* 大间距 */
--macos-spacing-xl: 24px;  /* 超大间距 */
```

### 圆角系统
```css
--macos-radius-sm: 6px;    /* 小圆角 */
--macos-radius-md: 10px;   /* 中圆角 */
--macos-radius-lg: 14px;   /* 大圆角 */
--macos-radius-xl: 18px;   /* 超大圆角 */
```

### 颜色系统
```css
/* 背景 */
--macos-bg-primary: rgba(30, 30, 30, 0.85);
--macos-bg-secondary: rgba(40, 40, 40, 0.75);
--macos-bg-tertiary: rgba(50, 50, 50, 0.65);

/* 文字 */
--macos-text-primary: rgba(255, 255, 255, 0.95);
--macos-text-secondary: rgba(255, 255, 255, 0.65);
--macos-text-tertiary: rgba(255, 255, 255, 0.45);

/* 强调色 */
--macos-accent: #007AFF;
--macos-accent-hover: #0051D5;
```

## 🎯 应用示例

### 1. 主窗口
```css
.app-container {
  background: rgba(30, 30, 30, 0.85);
  backdrop-filter: blur(40px) saturate(180%);
  border-radius: 18px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
}
```

### 2. 搜索栏
```css
.search-input {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 10px;
}

.search-input:focus {
  border-color: #007AFF;
  box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.15);
}
```

### 3. 卡片
```css
.card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  transition: all 200ms cubic-bezier(0.4, 0.0, 0.2, 1);
}

.card:hover {
  background: rgba(255, 255, 255, 0.08);
  transform: translateY(-3px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}
```

### 4. 按钮
```css
.button {
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  transition: all 150ms cubic-bezier(0.4, 0.0, 0.2, 1);
}

.button:hover {
  background: rgba(255, 255, 255, 0.1);
  transform: translateY(-1px);
}

.button:active {
  transform: scale(0.95);
}
```

## 📚 使用的 CSS 类

### 通用类
- `.macos-vibrancy` - 毛玻璃效果
- `.macos-button` - 标准按钮
- `.macos-button-primary` - 主要按钮
- `.macos-input` - 输入框
- `.macos-card` - 卡片
- `.macos-divider` - 分隔线
- `.macos-scrollbar` - 滚动条

### 布局类
- `.macos-grid` - 网格布局
- `.macos-flex` - 弹性布局
- `.macos-flex-center` - 居中布局

### 文字类
- `.macos-text-title` - 标题
- `.macos-text-subtitle` - 副标题
- `.macos-text-body` - 正文
- `.macos-text-caption` - 说明文字

### 动画类
- `.macos-fade-in` - 淡入动画
- `.macos-scale-in` - 缩放进入动画

## 🎨 设计原则

### 1. 层次感
- 使用半透明背景创建层次
- 通过阴影区分不同层级
- 悬停时增强层次感

### 2. 流畅性
- 所有交互都有动画反馈
- 使用 macOS 标准缓动函数
- 动画时长适中（150-250ms）

### 3. 一致性
- 统一的圆角系统
- 统一的间距系统
- 统一的颜色系统
- 统一的动画效果

### 4. 精致感
- 细腻的边框（1px）
- 适度的透明度
- 柔和的阴影
- 流畅的动画

## 🚀 效果预览

### 启动应用
```bash
pnpm tauri dev
```

### 预期效果
1. ✅ 窗口有圆角和阴影
2. ✅ 背景是半透明毛玻璃效果
3. ✅ 所有元素都有流畅的动画
4. ✅ 悬停时有明显的反馈
5. ✅ 整体感觉精致、现代

## 📖 参考资料

- [Apple Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/macos)
- [SF Pro Font](https://developer.apple.com/fonts/)
- [macOS Design Resources](https://developer.apple.com/design/resources/)

## 🎉 完成！

现在你的应用完全符合 macOS 设计规范，拥有：
- ✅ 毛玻璃效果
- ✅ 流畅的动画
- ✅ 精致的细节
- ✅ 原生的质感

就像真正的 macOS 原生应用一样！
