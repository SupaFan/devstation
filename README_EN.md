# DevStation

A macOS desktop frontend project manager built with Tauri 2 + Vue 3 + TypeScript + Naive UI. Manage dev servers, builds, and Git operations for all your frontend projects in one place.

[中文文档](./README.md)

![DevStation Screenshot](./screenshot.png)

## Features

### Project Management
- **Add Projects** — Import one or multiple project folders, auto-detects `package.json`
- **Scan Workspace** — Select a workspace folder to auto-discover all sub-projects
- **Framework Detection** — Auto-detects Vue, React, Angular, Svelte, Nuxt, Next.js
- **Version Display** — Shows project version from `package.json`
- **Git Info** — Auto-fetches branch name and remote URL
- **Favorites** — Star projects for quick filtering
- **Custom Sort** — Double-click the sort number to set a custom order
- **Search & Filter** — Search by project name, directory, path, or framework
- **Batch Delete** — Select multiple projects and remove them at once

### Dev & Build
- **One-Click Dev** — Click the play button to launch a dev server in a new terminal window
- **One-Click Build** — Run the build command in a new terminal window
- **Run/Stop Toggle** — Running projects show a stop button; click to kill the port process
- **Custom Commands** — Override dev/build commands per project
- **Run Any Script** — Pick any script from `package.json` via the dropdown menu

### Tool Integration
- **Open in IDE** — Launch project in Trae / VS Code with one click
- **Open in Terminal** — Open Terminal.app at the project directory
- **Open in Finder** — Reveal project folder in Finder
- **Git Pull** — Pull single or multiple projects at once
- **Outdated Dependencies** — Check for outdated npm packages

### View Modes
- **Table View** — High information density, ideal for managing many projects
- **Card View** — Visual overview, great for quick browsing

## Installation

### Download
Download the `.dmg` installer from the Releases page and drag to Applications.

If you see a "damaged" warning after install:
```bash
sudo xattr -rd com.apple.quarantine /Applications/DevStation.app
```

### Build from Source

**Prerequisites:**
- Node.js >= 18
- pnpm
- Rust (via rustup)
- Xcode Command Line Tools

```bash
# Clone the repo
git clone https://github.com/your-repo/devstation.git
cd devstation

# Install dependencies
pnpm install

# Development mode
pnpm tauri dev

# Production build
pnpm tauri build
```

Build output is in `src-tauri/target/release/bundle/`:
- `dmg/` — .dmg installer
- `macos/` — .app bundle

Optional flags:
```bash
pnpm tauri build --bundles app   # .app only
pnpm tauri build --no-sign       # Skip code signing
```

## Usage

### Getting Started
1. Open DevStation
2. Click "Add Project" in the top-right corner to select project directories
3. Or click the folder icon to scan an entire workspace

### Quick Reference
| Action | How |
|--------|-----|
| Run Dev | Click the green play button |
| Stop Dev | Button turns red while running — click to stop |
| Build | Click the yellow build button |
| Open in IDE | Click the blue open button |
| Custom Command | `...` menu → Configure Commands |
| Change Sort Order | Double-click the sort number, type new value, Enter |
| Rename Project | Double-click the project name, type new name, Enter |
| Toggle Favorite | Click the star icon |
| Search | Type in the search bar |
| Filter Favorites | Select "Favorites" from the dropdown |
| Check Dependencies | `...` menu → Check Outdated Dependencies |

### Global Settings
Click the gear icon in the top-right corner:
- Workspace folder management
- IDE name (default: Trae)
- Package manager (default: pnpm)
- Dev script name (default: dev)
- Build script name (default: build)

## Tech Stack

- **Frontend**: Vue 3 + TypeScript + Naive UI + Pinia
- **Backend**: Rust + Tauri 2
- **Storage**: Local JSON file (`~/.devstation/config.json`)

## License

MIT
