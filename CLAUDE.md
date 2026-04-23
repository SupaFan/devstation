# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DevStation is a macOS desktop app for managing frontend projects. Built with **Tauri 2** (Rust backend) + **Vue 3 + Naive UI** (frontend). It manages dev/build, Git operations, IDE launching, and dependency checks across multiple projects from a single UI.

## Commands

```bash
pnpm install          # Install frontend dependencies
pnpm tauri dev        # Dev mode (starts Vite on :1420 + Rust backend with hot reload)
pnpm tauri build      # Production build (generates .dmg/.app in src-tauri/target/release/bundle/)
pnpm build            # Frontend-only build (vue-tsc type check + vite build)
```

There are no tests or linter configured.

## Architecture

### Frontend (`src/`)

- **Entry**: `main.ts` → `App.vue` (Pinia setup, Naive UI config provider with dark theme toggle)
- **State**: Single Pinia store in `stores/app.ts` — holds all config, project list, UI state. All Tauri `invoke()` calls go through this store.
- **Views**: `views/MainView.vue` (project list/table/cards) and `views/SettingsView.vue` (global config), toggled via `store.currentView`
- **Components**: `components/ProjectTable.vue` — the main table/card rendering
- **Types**: `types/index.ts` mirrors the Rust models exactly (`Project`, `AppConfig`, `OutdatedDep`)

### Backend (`src-tauri/src/`)

- **`lib.rs`** — Plugin registration (opener, dialog, shell, fs) and command handler registration
- **`models.rs`** — `Project`, `AppConfig`, `OutdatedDep` structs with Serialize/Deserialize
- **`commands/`** — Four modules, each exposing `#[command]` functions:
  - `project.rs` — Config CRUD, folder scanning, project detection (framework, port, scripts from package.json)
  - `git.rs` — Git remote/branch retrieval, batch pull, outdated dep check via `pnpm outdated`
  - `runner.rs` — Dev/build/script execution in new Terminal.app windows via osascript
  - `system.rs` — Open in IDE (Trae/VS Code/Cursor), Terminal, Finder, port detection via lsof

### Data Flow

- Config persisted as JSON at `~/.devstation/config.json`
- Frontend calls Rust via Tauri's `invoke()` — no REST API
- Commands that mutate config receive the full `AppConfig` from frontend, update it, save to disk, and return the updated config
- Project IDs are hashes of the project directory path

### Key Patterns

- **Port detection**: Cascading — scripts → config files (vite.config.ts, nuxt.config.ts) → framework defaults (Vue/React/Svelte→5173, Nuxt/Next→3000, Angular→4200)
- **Framework detection**: Based on presence of framework packages in dependencies/devDependencies
- **Terminal integration**: macOS uses AppleScript (`osascript`) to open Terminal.app; Windows uses `cmd /c start`
- **Running state**: Frontend tracks `runningPorts` Set, checked via `lsof` on Rust side

## Tauri Config

- Vite dev server: `localhost:1420`
- Window: 1200x800, min 900x600
- Bundle identifier: `com.remsupa.devstation`
- Plugins: opener, dialog, shell, fs (all registered in `capabilities/default.json`)
- Rust crate name: `devstation_lib`
