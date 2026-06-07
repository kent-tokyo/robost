# Robost Editor

A modern Visual RPA (Robotic Process Automation) scenario editor built with Electron and React, powered by a Rust backend CLI.

## Features

- **Visual Node-Based Editor**: Drag-and-drop canvas for scenario design using ReactFlow
- **YAML Code View**: Edit scenarios as YAML with Monaco Editor
- **Real-Time Execution Logs**: Monitor step progress with live streaming
- **Property Inspector**: Edit step parameters with context-aware forms
- **Template Gallery**: Pre-built scenario templates for common tasks
- **Multi-Language Support**: English, Japanese, Chinese (i18n)
- **Light/Dark Themes**: VS Code-inspired design system
- **File Management**: New, Open, Save, Save As with recent files
- **AI Integration**: Anthropic Claude for step suggestions
- **Cross-Platform**: macOS (Intel/ARM) and Windows x64

## Quick Start

### Prerequisites

- Node.js 18+ and npm
- Rust toolchain (for building the RPA CLI backend)

### Installation

```bash
# Clone repository
git clone https://github.com/yourusername/robost.git
cd robost/apps/electron-editor

# Install dependencies
npm install
```

### Development

```bash
# Start Electron dev server with hot reload
npm run start
```

The app will open at `http://localhost:3000` with the Electron window.

### Building for Distribution

```bash
# Build RPA binaries for macOS and Windows
npm run build:rpa

# Package for your current platform
npm run make

# Or platform-specific:
npm run make:mac      # macOS DMG + ZIP
npm run make:win      # Windows NSIS installer + ZIP
```

Packaged apps will be in `out/make/`.

## Project Structure

```
apps/electron-editor/
├── src/
│   ├── main/                    # Electron main process
│   │   ├── index.ts             # Main window setup
│   │   ├── preload.ts           # IPC bridge
│   │   └── rpaManager.ts        # RPA process lifecycle
│   ├── renderer/                # React frontend
│   │   ├── components/          # UI components
│   │   │   ├── ActivityBar.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Editor.tsx       # Canvas/Code/List views
│   │   │   ├── Canvas.tsx       # ReactFlow node editor
│   │   │   ├── Inspector.tsx    # Property panel
│   │   │   ├── ProgressPanel.tsx
│   │   │   ├── StatusBar.tsx
│   │   │   └── SettingsPanel.tsx
│   │   ├── hooks/
│   │   │   ├── useRpaServer.ts  # RPA execution
│   │   │   └── useFileManager.ts # File operations
│   │   ├── store/               # Zustand state management
│   │   │   ├── editorStore.ts
│   │   │   ├── scenarioStore.ts
│   │   │   ├── runStore.ts
│   │   │   └── settingsStore.ts
│   │   ├── types/
│   │   │   └── stepSchema.ts    # Step type definitions
│   │   ├── locales/             # i18n translations
│   │   │   ├── en.json
│   │   │   ├── ja.json
│   │   │   └── zh.json
│   │   ├── i18n.ts              # i18next setup
│   │   ├── App.tsx              # Root component
│   │   └── globals.css          # Theme variables
│   └── index.html
├── assets/
│   ├── rpa/                     # Platform-specific RPA binaries
│   │   ├── darwin-arm64/rpa
│   │   ├── darwin-x64/rpa
│   │   └── win32-x64/rpa.exe
│   └── entitlements.plist       # macOS security entitlements
├── forge.config.js              # Electron Forge config
├── webpack.*.config.js          # Build config
├── package.json
├── tsconfig.json
├── PACKAGING.md                 # Detailed packaging guide
└── README.md (this file)
```

## Architecture

```
Electron Editor (React + TypeScript)
    ↓ Child Process + HTTP/IPC
Robost CLI (Rust) -- --serve 127.0.0.1:0
    ↓ Library calls
Robost Core Engine
    ├── Vision (NCC image matching)
    ├── Windows Automation (UIA)
    ├── Web Automation (WebDriver)
    └── Standard Library (Excel, PDF, etc.)
```

## Key Technologies

- **Frontend**: React 19, TypeScript, Zustand (state), ReactFlow (canvas)
- **Editor**: Monaco Editor (YAML) + react-i18next (i18n)
- **Desktop**: Electron 42+ (main process), Electron Forge (packaging)
- **Backend**: Robost CLI (Rust), HTTP Server (SSE streaming)
- **Styling**: CSS Variables (VS Code palette), TailwindCSS optional

## State Management

The app uses Zustand with immer middleware:

- **editorStore**: `scenarioPath`, `selectedNodeId`, undo/redo history
- **scenarioStore**: Scenario data (steps, variables), canvas layout
- **runStore**: Execution state (isRunning, logs, currentStep)
- **settingsStore**: User preferences (theme, locale, API keys, recent files)

All stores persist to localStorage.

## RPA Binary Integration

The editor spawns the `rpa` CLI with `--serve` flag:

```bash
rpa run scenario.yaml --serve 127.0.0.1:0
```

The server outputs `PORT=XXXX` to stdout, which the editor connects to via HTTP/SSE for real-time progress.

## Translations

Edit `src/renderer/locales/*.json` to add/update strings. The i18n system uses i18next with lazy loading.

Namespaces: `activityBar`, `sidebar`, `editor`, `inspector`, `progressPanel`, `statusBar`, `settings`, `common`

## Settings

User settings are stored in Zustand with localStorage persistence:

- **theme**: `'dark'` (default) or `'light'`
- **locale**: `'en'`, `'ja'`, `'zh'`
- **autoSave**: Enable/disable auto-save
- **autoSaveInterval**: Auto-save frequency in ms
- **apiKeyOpenAI** / **apiKeyAnthropic**: For AI features
- **recentFiles**: Last 5 opened files with timestamps

## Keyboard Shortcuts

- **Cmd+N** (macOS) / **Ctrl+N** (Windows): New scenario
- **Cmd+O** / **Ctrl+O**: Open scenario
- **Cmd+S** / **Ctrl+S**: Save scenario
- **Cmd+Z** / **Ctrl+Z**: Undo
- **Cmd+Shift+Z** / **Ctrl+Shift+Z**: Redo
- **F12**: Developer Tools

## Contributing

1. Fork the repo
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit changes: `git commit -m 'Add feature'`
4. Push: `git push origin feature/my-feature`
5. Open a Pull Request

## License

MIT

## Support

- GitHub Issues: Report bugs or request features
- Discussions: Ask questions and share ideas
- Documentation: See PACKAGING.md for detailed build/distribution info
