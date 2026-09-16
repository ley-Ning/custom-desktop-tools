import { openDB, DBSchema, IDBPDatabase } from 'idb';
import { invoke } from '@tauri-apps/api/core';
import type { Plugin } from '../types';

// 供组件直接从 "../db" 导入 Plugin 类型
export type { Plugin };

// 数据库 Schema 定义
// 剪贴板/备忘录数据已迁移到 Rust 后端存储（clipboard.json / memos.json），
// IndexedDB 仅保留插件注册表
interface MyUToolsDB extends DBSchema {
    plugins: {
        key: string;
        value: Plugin;
    };
}

const DB_NAME = 'my-utools-db';
const DB_VERSION = 2;

let dbInstance: IDBPDatabase<MyUToolsDB> | null = null;

// 初始化数据库
export async function initDB(): Promise<IDBPDatabase<MyUToolsDB>> {
    if (dbInstance) return dbInstance;

    dbInstance = await openDB<MyUToolsDB>(DB_NAME, DB_VERSION, {
        upgrade(db) {
            // 创建插件表
            if (!db.objectStoreNames.contains('plugins')) {
                db.createObjectStore('plugins', { keyPath: 'id' });
            }
        },
    });

    // 首次初始化时添加默认插件
    const count = await dbInstance.count('plugins');
    if (count === 0) {
        await initDefaultPlugins(dbInstance);
    }

    return dbInstance;
}

// 初始化默认插件
async function initDefaultPlugins(db: IDBPDatabase<MyUToolsDB>) {
    const defaultPlugins: Plugin[] = [
        {
            id: 'clipboard',
            name: '剪贴板历史',
            icon: '📋',
            gradient: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            keywords: ['剪贴板', 'clipboard', '复制', '粘贴', 'copy', 'paste', '历史', 'jiantieba', 'jtb'],
            description: '自动记录剪贴板历史，支持搜索和收藏',
            enabled: true,
            builtin: true,
            version: '1.0.0',
            author: 'My uTools',
        },
        {
            id: 'memo',
            name: '备忘快贴',
            icon: '📝',
            gradient: 'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
            keywords: ['备忘录', 'memo', '笔记', 'note', '记事', 'beiwanglu', 'bwl', 'biji'],
            description: '快速记录备忘录，支持标签和置顶',
            enabled: true,
            builtin: true,
            version: '1.0.0',
            author: 'My uTools',
        },
        {
            id: 'json',
            name: 'JSON 编辑器',
            icon: '📊',
            gradient: 'linear-gradient(135deg, #fa709a 0%, #fee140 100%)',
            keywords: ['json', '编辑器', '格式化', 'format', 'bianjiqi', 'geshihua'],
            description: '格式化和编辑 JSON 数据',
            enabled: true,
            builtin: true,
            version: '1.0.0',
            author: 'My uTools',
        },
        {
            id: 'ai',
            name: 'AI 对话',
            icon: '🤖',
            gradient: 'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)',
            keywords: ['ai', '对话', '聊天', 'chat', 'gpt', 'duihua', 'liaotian'],
            description: '与 AI 进行智能对话，支持多会话与 Markdown 渲染',
            enabled: true,
            builtin: true,
            version: '1.0.0',
            author: 'My uTools',
        },
        {
            id: 'translator',
            name: '聚合翻译',
            icon: '🌐',
            gradient: 'linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)',
            keywords: ['翻译', 'translate', 'fanyi', 'fy', '英文', '中文', 'english', '日语', 'riyu'],
            description: '多引擎聚合翻译（MyMemory / Google / AI）',
            enabled: true,
            builtin: true,
            version: '1.0.0',
            author: 'My uTools',
        },
        {
            id: 'calc',
            name: '计算稿纸',
            icon: '🧮',
            gradient: 'linear-gradient(135deg, #30cfd0 0%, #330867 100%)',
            keywords: ['计算', 'calculator', 'calc', '数学', 'jisuan', 'jisuanqi', '求值'],
            description: '稿纸式计算器，支持变量与函数',
            enabled: true,
            builtin: true,
            version: '1.0.0',
            author: 'My uTools',
        },
        {
            id: 'timestamp',
            name: '时间戳转换',
            icon: '🕐',
            gradient: 'linear-gradient(135deg, #a8edea 0%, #fed6e3 100%)',
            keywords: ['时间', 'time', 'timestamp', '日期', 'shijian', 'sj', 'shijianchuo'],
            description: '时间戳与日期互转',
            enabled: true,
            builtin: true,
            version: '1.0.0',
            author: 'My uTools',
        },
    ];

    const tx = db.transaction('plugins', 'readwrite');
    await Promise.all([
        ...defaultPlugins.map(plugin => tx.store.add(plugin)),
        tx.done,
    ]);
}

// 获取所有插件
export async function getAllPlugins(): Promise<Plugin[]> {
    const db = await initDB();
    return db.getAll('plugins');
}

// 获取已启用的插件
// 注：不使用 IDB 索引查询（boolean 不是合法的 IDB key，getAllFromIndex 会抛 DataError）
export async function getEnabledPlugins(): Promise<Plugin[]> {
    const db = await initDB();
    const all = await db.getAll('plugins');
    return all.filter(plugin => plugin.enabled);
}

// 获取单个插件
export async function getPlugin(id: string): Promise<Plugin | undefined> {
    const db = await initDB();
    return db.get('plugins', id);
}

// 添加插件
export async function addPlugin(plugin: Plugin): Promise<void> {
    const db = await initDB();
    await db.add('plugins', plugin);
}

// 更新插件
export async function updatePlugin(plugin: Plugin): Promise<void> {
    const db = await initDB();
    await db.put('plugins', plugin);
}

// 删除插件
export async function deletePlugin(id: string): Promise<void> {
    const db = await initDB();
    const plugin = await db.get('plugins', id);

    // 不允许删除内置插件
    if (plugin?.builtin) {
        throw new Error('不能删除内置插件');
    }

    await db.delete('plugins', id);
}

// 切换插件启用状态
export async function togglePluginEnabled(id: string): Promise<void> {
    const db = await initDB();
    const plugin = await db.get('plugins', id);

    if (plugin) {
        plugin.enabled = !plugin.enabled;
        await db.put('plugins', plugin);
    }
}

// 搜索插件（根据关键词）
export async function searchPlugins(query: string): Promise<Plugin[]> {
    const db = await initDB();
    const allPlugins = await db.getAll('plugins');

    const queryLower = query.toLowerCase().trim();
    if (!queryLower) return [];

    return allPlugins.filter(plugin => {
        if (!plugin.enabled) return false;

        return plugin.keywords.some(keyword => {
            const keywordLower = keyword.toLowerCase();
            return keywordLower.includes(queryLower) || queryLower.includes(keywordLower);
        });
    });
}

// ===== 剪贴板历史（数据存放在 Rust 后端 clipboard.json） =====

/** 剪贴板条目（camelCase，与组件约定一致；timestamp 为毫秒） */
export interface ClipboardItem {
    id: string;
    content: string;
    contentType: string;
    timestamp: number;
    favorite: boolean;
    filePath?: string;
    fileSize?: number;
}

/** Rust 后端的 snake_case 原始结构 */
interface RawClipboardItem {
    id: string;
    content: string;
    content_type: string;
    timestamp: number; // 秒
    favorite: boolean;
    file_path?: string;
    file_size?: number;
}

function mapClipboardItem(raw: RawClipboardItem): ClipboardItem {
    return {
        id: raw.id,
        content: raw.content,
        contentType: raw.content_type,
        // 组件 formatTime 按毫秒处理，后端存的是秒
        timestamp: raw.timestamp * 1000,
        favorite: raw.favorite,
        filePath: raw.file_path,
        fileSize: raw.file_size,
    };
}

// 获取剪贴板历史
export async function getClipboardHistory(): Promise<ClipboardItem[]> {
    try {
        const raw = await invoke<RawClipboardItem[]>("get_clipboard_history");
        return (raw || []).map(mapClipboardItem);
    } catch (e) {
        console.error("Failed to load clipboard history:", e);
        return [];
    }
}

// 添加剪贴板项
export async function addClipboardItem(item: { content: string; contentType: string }): Promise<ClipboardItem | null> {
    try {
        const raw = await invoke<RawClipboardItem>("add_clipboard_item", {
            content: item.content,
            contentType: item.contentType,
        });
        return mapClipboardItem(raw);
    } catch (e) {
        console.error("Failed to add clipboard item:", e);
        return null;
    }
}

// 删除剪贴板项
export async function deleteClipboardItem(id: string): Promise<void> {
    await invoke("delete_clipboard_item", { id });
}

// 清空剪贴板历史
export async function clearClipboardHistory(): Promise<void> {
    await invoke("clear_clipboard_history");
}

// 切换剪贴板项收藏状态
export async function toggleClipboardFavorite(id: string): Promise<void> {
    await invoke("toggle_clipboard_favorite", { id });
}

// ===== 备忘录（数据存放在 Rust 后端 memos.json） =====

/** 备忘录条目（字段与 Rust MemoItem 一致） */
export interface MemoItem {
    id: string;
    title: string;
    content: string;
    tags: string[];
    timestamp: number;
    updated_at: number;
    pinned: boolean;
}

// 获取备忘录列表
export async function getMemos(): Promise<MemoItem[]> {
    try {
        const items = await invoke<MemoItem[]>("get_memos");
        return items || [];
    } catch (e) {
        console.error("Failed to load memos:", e);
        return [];
    }
}

// 添加备忘录
export async function addMemo(input: { title: string; content: string; tags: string[] }): Promise<MemoItem | null> {
    try {
        return await invoke<MemoItem>("add_memo", input);
    } catch (e) {
        console.error("Failed to add memo:", e);
        return null;
    }
}

// 更新备忘录
export async function updateMemo(id: string, input: { title: string; content: string; tags: string[] }): Promise<void> {
    await invoke("update_memo", { id, ...input });
}

// 删除备忘录
export async function deleteMemo(id: string): Promise<void> {
    await invoke("delete_memo", { id });
}

// 切换备忘录置顶状态
export async function toggleMemoPinned(id: string): Promise<void> {
    await invoke("toggle_memo_pin", { id });
}

// 开发工具：清除数据库（仅用于开发调试）
export async function clearDatabase(): Promise<void> {
    if (dbInstance) {
        dbInstance.close();
        dbInstance = null;
    }
    const { deleteDB } = await import('idb');
    await deleteDB(DB_NAME);
    console.log('Database cleared. Please refresh the page.');
}

// 开发工具：在控制台暴露清除函数
if (typeof window !== 'undefined') {
    (window as any).clearDB = clearDatabase;
}
