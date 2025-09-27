# Tauri UI Implementation Plan

A plan for implementing a minimal Tauri UI wrapper for the existing CLI program using React/Vite.

## Project Structure

```
├── Cargo.toml (update with Tauri deps)
├── src/ (existing Rust CLI code)
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   └── icons/
├── src-ui/ (React frontend)
│   ├── package.json
│   ├── vite.config.ts
│   ├── index.html
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── components/
│   │   │   ├── DeviceStatus.tsx
│   │   │   ├── PackageList.tsx
│   │   │   ├── RepositorySelector.tsx
│   │   │   └── ConsoleOutput.tsx
│   │   └── hooks/
│   │       └── useTauri.ts
│   └── public/
```

## 1. Tauri Setup & Dependencies

### Root Cargo.toml Updates
Add Tauri workspace and dependencies:
```toml
[workspace]
members = ["src-tauri"]

# Add to existing dependencies
tauri = { version = "1.5", features = ["api-all"] }
```

### src-tauri/Cargo.toml
```toml
[package]
name = "penumbra-installer-ui"
version = "0.1.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "1.5", features = [] }

[dependencies]
tauri = { version = "1.5", features = ["api-all", "shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }

# Reference existing CLI crate
penumbra-installer = { path = ".." }
```

## 2. Core UI Components

### Device Status Panel
- Connection indicator (Connected/Disconnected/Multiple Devices)
- Device information display
- ADB connection status
- Error messages for connection issues

### App Version Display
- List currently installed packages matching config patterns
- Show version information for each app
- Refresh button to update package list
- Loading states

### Installation Controls
- Repository selection with checkboxes
- Install Selected / Install All buttons
- Progress indicators during installation
- Cancel installation option

### Console Output
- Real-time installation progress
- Scrollable output window
- Clear console button
- Export logs functionality

## 3. Tauri Commands (Rust Backend)

```rust
// src-tauri/src/main.rs

use penumbra_installer::{AdbManager, InstallationEngine, ConfigLoader};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DeviceInfo {
    connected: bool,
    device_count: usize,
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PackageInfo {
    package_name: String,
    version: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Repository {
    name: String,
    owner: String,
    repo: String,
    description: Option<String>,
}

// Device management commands
#[tauri::command]
async fn check_device_connection() -> Result<DeviceInfo, String> {
    // Wrap existing AdbManager::connect() logic
}

#[tauri::command]
async fn list_installed_packages() -> Result<Vec<PackageInfo>, String> {
    // Use AdbManager to list packages matching config patterns
}

// Installation operations
#[tauri::command]
async fn install_repositories(
    repos: Vec<String>,
    window: tauri::Window,
) -> Result<String, String> {
    // Wrap existing InstallationEngine logic
    // Use window.emit() for progress updates
}

#[tauri::command]
async fn get_available_repositories() -> Result<Vec<Repository>, String> {
    // Load config and return repository list
}

#[tauri::command]
async fn cancel_installation() -> Result<(), String> {
    // Implementation for canceling ongoing installation
}
```

## 4. React Components Architecture

### App.tsx - Main Application
```typescript
interface AppState {
    deviceConnected: boolean;
    deviceInfo: DeviceInfo | null;
    installedPackages: PackageInfo[];
    availableRepos: Repository[];
    selectedRepos: string[];
    installing: boolean;
    consoleOutput: string[];
    error: string | null;
}
```

### Component Responsibilities
- **DeviceStatus.tsx**: Device connection status and info
- **PackageList.tsx**: Installed packages with refresh capability
- **RepositorySelector.tsx**: Multi-select repository interface
- **ConsoleOutput.tsx**: Real-time installation progress display

## 5. Tauri Configuration

### tauri.conf.json
```json
{
  "package": {
    "productName": "Penumbra Installer",
    "version": "0.1.0"
  },
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist",
    "withGlobalTauri": false
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "window": {
        "all": false,
        "close": true,
        "hide": true,
        "show": true,
        "maximize": true,
        "minimize": true,
        "unmaximize": true,
        "unminimize": true,
        "startDragging": true
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.penumbra.installer",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    },
    "security": {
      "csp": null
    },
    "windows": [
      {
        "fullscreen": false,
        "height": 700,
        "resizable": true,
        "title": "Penumbra Installer",
        "width": 900,
        "minWidth": 600,
        "minHeight": 500
      }
    ]
  }
}
```

## 6. Frontend Dependencies

### src-ui/package.json
```json
{
  "name": "penumbra-installer-ui",
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@tauri-apps/api": "^1.5.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@vitejs/plugin-react": "^4.0.0",
    "typescript": "^5.0.0",
    "vite": "^4.4.0"
  }
}
```

### vite.config.ts
```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(async () => ({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
}));
```

## 7. Tauri API Integration

### useTauri.ts Hook
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

export interface UseTauriAPI {
  checkDeviceConnection: () => Promise<DeviceInfo>;
  listInstalledPackages: () => Promise<PackageInfo[]>;
  installRepositories: (repos: string[]) => Promise<string>;
  getAvailableRepositories: () => Promise<Repository[]>;
  cancelInstallation: () => Promise<void>;
}

export const useTauri = (): UseTauriAPI => {
  return {
    checkDeviceConnection: () => invoke('check_device_connection'),
    listInstalledPackages: () => invoke('list_installed_packages'),
    installRepositories: (repos: string[]) => invoke('install_repositories', { repos }),
    getAvailableRepositories: () => invoke('get_available_repositories'),
    cancelInstallation: () => invoke('cancel_installation'),
  };
};

// Event listening for real-time updates
export const useInstallationProgress = (callback: (message: string) => void) => {
  useEffect(() => {
    const unlisten = listen('installation_progress', (event) => {
      callback(event.payload as string);
    });

    return () => {
      unlisten.then(f => f());
    };
  }, [callback]);
};
```

## 8. UI Layout Design

### Minimal UI Layout
```
┌─────────────────────────────────────────────────┐
│ Penumbra Installer                    [_][□][×] │
├─────────────────────────────────────────────────┤
│ Device Status                                   │
│ ● Connected - Device ready for installation     │
│ Device: SM-G991B (Android 13)        [Refresh] │
├─────────────────────────────────────────────────┤
│ Installed Packages                              │
│ • com.penumbra.launcher (v2.1.0)               │
│ • com.penumbra.settings (v1.5.2)               │
│ • ai.humane.cosmos (v3.0.1)           [Refresh] │
├─────────────────────────────────────────────────┤
│ Available Repositories                          │
│ ☑ PenumbraOS Core                              │
│ ☐ Humane Cosmos                                │
│ ☐ Development Tools                            │
│ [Install Selected] [Install All]    [Cancel]   │
├─────────────────────────────────────────────────┤
│ Installation Progress              [Clear][⤓]   │
│ ┌─────────────────────────────────────────────┐ │
│ │ Starting PenumbraOS Core installation...   │ │
│ │ Downloading release assets...              │ │
│ │ ✓ Downloaded: launcher.apk                │ │
│ │ Installing APKs...                         │ │
│ │ ✓ Installed: launcher.apk                 │ │
│ │ Installation complete                      │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

## 9. Implementation Steps

1. **Initialize Tauri Project**
   - `cargo install tauri-cli`
   - Set up `src-tauri/` directory structure
   - Configure `tauri.conf.json`

2. **Refactor CLI Logic**
   - Move shared functionality to `lib.rs`
   - Create public APIs for Tauri commands
   - Maintain CLI binary compatibility

3. **Create Tauri Commands**
   - Implement async command wrappers
   - Add error handling and serialization
   - Set up event emission for progress updates

4. **Build React Frontend**
   - Initialize Vite + React + TypeScript
   - Create component structure
   - Implement Tauri API integration hooks

5. **Connect Frontend to Backend**
   - Test all Tauri commands
   - Implement real-time progress updates
   - Add proper error handling and loading states

6. **Polish and Test**
   - Add proper styling and responsive design
   - Test with actual device connections
   - Handle edge cases and error scenarios

## 10. Development Workflow

- **Development**: `npm run tauri dev`
  - React dev server runs on localhost:1420
  - Tauri window loads React app with hot reload
  - Rust changes require restart

- **Building**: `npm run tauri build`
  - Builds React app for production
  - Compiles Rust binary with Tauri
  - Generates platform-specific installers

- **Testing**:
  - Unit tests for Rust commands
  - React component testing
  - Integration testing with mock ADB connections

This plan maintains the existing CLI functionality while providing a modern, user-friendly GUI that directly leverages the same underlying Rust code.