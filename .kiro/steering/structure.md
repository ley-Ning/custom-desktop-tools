# Project Structure

## Root Layout

```
custom-desktop-tools/
├── src/                    # Vue frontend source
├── src-tauri/             # Rust backend source
├── public/                # Static assets
├── node_modules/          # Node dependencies
├── package.json           # Frontend dependencies
├── pnpm-workspace.yaml    # pnpm workspace config
├── vite.config.ts         # Vite configuration
└── tsconfig.json          # TypeScript configuration
```

## Frontend (`src/`)

```
src/
├── components/            # Vue components
│   ├── AiModelSettings.vue      # AI model configuration
│   ├── AppIcon.vue              # App icon display
│   ├── ClipboardPlugin.vue      # Clipboard history UI
│   ├── JsonEditorPlugin.vue     # JSON editor with CodeMirror
│   ├── McpSettings.vue          # MCP server settings
│   ├── MemoPlugin.vue           # Memo/notes UI
│   ├── PluginList.vue           # Search results list
│   ├── PluginMarket.vue         # Plugin marketplace UI
│   ├── SearchBar.vue            # Search input component
│   └── Settings.vue             # Settings panel
├── styles/                # Global styles
│   ├── main.css                 # Base styles
│   └── macos.css                # macOS-specific styles
├── types/                 # TypeScript type definitions
│   └── index.ts                 # Shared interfaces
├── App.vue                # Root component
├── main.ts                # Application entry point
└── vite-env.d.ts          # Vite type declarations
```

## Backend (`src-tauri/`)

```
src-tauri/
├── src/
│   ├── lib.rs             # Core logic (commands, state, plugins)
│   └── main.rs            # Entry point (minimal, calls lib.rs)
├── capabilities/          # Tauri permission manifests
├── gen/schemas/           # Generated schemas
├── icons/                 # App icons (PNG, SVG)
├── target/                # Rust build artifacts
├── Cargo.toml             # Rust dependencies
├── tauri.conf.json        # Tauri app configuration
└── build.rs               # Build script
```

## Key Architectural Patterns

### Frontend

- **Composition API**: All components use `<script setup>` syntax
- **Reactive State**: `ref()` and `computed()` for reactivity
- **Event Communication**: Tauri events for backend-to-frontend messaging
- **State Persistence**: localStorage for UI state, Tauri store for data

### Backend

- **Command Pattern**: `#[command]` functions exposed to frontend via `invoke()`
- **State Management**: `AppState` with `Arc<Mutex<>>` for shared state
- **Plugin Data**: Separate JSON stores per plugin (clipboard, memos, etc.)
- **Event Emission**: `window.emit()` for backend-to-frontend events

### Component Organization

- **Plugin Components**: Self-contained with `@close` event for navigation
- **Settings Components**: Nested settings panels with left sidebar navigation
- **Shared Types**: Defined in `src/types/index.ts` and mirrored in Rust structs

## Configuration Files

- `tauri.conf.json`: Window size, permissions, bundle settings
- `vite.config.ts`: Dev server port (1420), Vue plugin
- `tsconfig.json`: Strict TypeScript with ES2020 target
- `Cargo.toml`: Rust dependencies and release optimizations
