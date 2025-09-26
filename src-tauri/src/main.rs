#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use penumbra_installer::{
    AdbManager, ConfigLoader, InstallerError, Repository,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DeviceInfo {
    connected: bool,
    device_count: usize,
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PackageInfo {
    package_name: String,
    version: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct RepositoryInfo {
    name: String,
    owner: String,
    repo: String,
    description: Option<String>,
}

impl From<&Repository> for RepositoryInfo {
    fn from(repo: &Repository) -> Self {
        Self {
            name: repo.name.clone(),
            owner: repo.owner.clone(),
            repo: repo.repo.clone(),
            description: None, // Repository struct doesn't have description field
        }
    }
}

// State for managing the installation process
struct AppState {
    installing: Mutex<bool>,
}

#[tauri::command]
async fn check_device_connection() -> Result<DeviceInfo, String> {
    match AdbManager::connect().await {
        Ok(_) => Ok(DeviceInfo {
            connected: true,
            device_count: 1,
            error_message: None,
        }),
        Err(InstallerError::NoDevice) => Ok(DeviceInfo {
            connected: false,
            device_count: 0,
            error_message: Some("No Android device connected. Please connect a device and enable USB debugging.".to_string()),
        }),
        Err(InstallerError::MultipleDevices) => Ok(DeviceInfo {
            connected: false,
            device_count: 2, // Indicating multiple devices
            error_message: Some("Multiple devices connected. Please connect exactly one device.".to_string()),
        }),
        Err(e) => Ok(DeviceInfo {
            connected: false,
            device_count: 0,
            error_message: Some(format!("ADB connection failed: {}", e)),
        }),
    }
}

#[tauri::command]
async fn list_installed_packages() -> Result<Vec<PackageInfo>, String> {
    // For now, return a simple response to test the UI
    // TODO: Implement actual ADB package listing
    Ok(vec![
        PackageInfo {
            package_name: "com.penumbra.launcher".to_string(),
            version: Some("2.1.0".to_string()),
        },
        PackageInfo {
            package_name: "com.humane.cosmos".to_string(),
            version: Some("1.5.2".to_string()),
        },
    ])
}

#[tauri::command]
async fn get_available_repositories() -> Result<Vec<RepositoryInfo>, String> {
    let config = ConfigLoader::load_builtin("penumbra")
        .map_err(|e| format!("Failed to load config: {}", e))?;

    let repos: Vec<RepositoryInfo> = config
        .repositories
        .iter()
        .map(|repo| repo.into())
        .collect();

    Ok(repos)
}

#[tauri::command]
async fn install_repositories(
    repos: Vec<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Check if already installing
    {
        let mut installing = state.installing.lock().unwrap();
        if *installing {
            return Err("Installation already in progress".to_string());
        }
        *installing = true;
    }

    // For now, simulate installation progress
    let _ = app.emit("installation_progress", "Starting installation...");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let selected_repos = if repos.is_empty() { "all repositories" } else { "selected repositories" };
    let _ = app.emit("installation_progress", format!("Installing {}...", selected_repos));
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let _ = app.emit("installation_progress", "Installation completed successfully!");

    // Reset installing state
    {
        let mut installing = state.installing.lock().unwrap();
        *installing = false;
    }

    Ok("Installation completed successfully".to_string())
}

#[tauri::command]
async fn cancel_installation(state: State<'_, AppState>) -> Result<(), String> {
    // For now, we just reset the installing state
    // TODO: Implement proper cancellation mechanism
    {
        let mut installing = state.installing.lock().unwrap();
        *installing = false;
    }
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            installing: Mutex::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            check_device_connection,
            list_installed_packages,
            get_available_repositories,
            install_repositories,
            cancel_installation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}