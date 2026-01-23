# Technology Stack

## Frontend

- **Framework**: Vue 3 with Composition API (`<script setup>`)
- **Language**: TypeScript with strict mode enabled
- **Build Tool**: Vite 5
- **UI Library**: Custom components with lucide-vue-next icons
- **Code Editor**: CodeMirror 6 for JSON editing

## Backend

- **Runtime**: Tauri 2 (Rust-based)
- **Language**: Rust 2021 edition
- **Key Dependencies**:
  - `tauri-plugin-global-shortcut`: Global hotkey registration
  - `tauri-plugin-store`: Persistent data storage
  - `tauri-plugin-shell`: Shell command execution
  - `serde`/`serde_json`: Serialization
  - `tokio`: Async runtime
  - `chrono`: Timestamp handling
  - `uuid`: Unique ID generation

## Package Management

- **Frontend**: pnpm (workspace-enabled)
- **Backend**: Cargo

## Common Commands

```bash
# Development
pnpm install              # Install dependencies
pnpm tauri dev           # Start dev server with hot reload

# Building
pnpm build               # Build frontend (TypeScript check + Vite build)
pnpm tauri build         # Build production app bundle

# Type Checking
vue-tsc --noEmit         # TypeScript validation (runs in build)
```

## Data Storage

- **Location**: `~/.local/share/com.lewen.my-utools/`
- **Format**: JSON files via tauri-plugin-store
  - `settings.json`: App settings (shortcuts, preferences)
  - `clipboard.json`: Clipboard history
  - `memos.json`: Memo items
  - `mcp.json`: MCP server configuration
  - `ai-config.json`: AI model configuration

## Platform-Specific

- **macOS Integration**: Uses `sips` for icon conversion, `pbcopy`/`pbpaste` for clipboard
- **Icon Extraction**: Converts `.icns` to base64 PNG for display
