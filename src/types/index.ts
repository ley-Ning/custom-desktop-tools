export interface App {
    id: string;
    name: string;
    path: string;
    icon?: string;
}

export interface Plugin {
    id: string;
    name: string;
    icon: string;
    gradient: string;
    keywords: string[];
    description: string;
    enabled: boolean;
    builtin: boolean; // 是否为内置插件
    version?: string;
    author?: string;
}

export interface PluginConfig {
    id: string;
    name: string;
    version: string;
    main: string;
    icon?: string;
    keywords?: string[];
    description?: string;
}
