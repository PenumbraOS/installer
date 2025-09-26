#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use penumbra_installer::{AdbManager, ConfigLoader, InstallerError, Repository};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tokio::task::spawn_blocking;

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
            error_message: Some(
                "No Android device connected. Please connect a device and enable USB debugging."
                    .to_string(),
            ),
        }),
        Err(InstallerError::MultipleDevices) => Ok(DeviceInfo {
            connected: false,
            device_count: 2, // Indicating multiple devices
            error_message: Some(
                "Multiple devices connected. Please connect exactly one device.".to_string(),
            ),
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
    spawn_blocking(move || {
        let package_names = vec![
            "com.penumbraos.pinitd",
            "com.penumbraos.bridge",
            "com.penumbraos.bridge_settings",
            "com.penumbraos.bridge_shell",
            "com.penumbraos.bridge_system",
            "com.penumbraos.mabl.pin",
            "com.penumbraos.plugins.aipinsystem",
            "com.penumbraos.plugins.demo",
            "com.penumbraos.plugins.googlesearch",
            "com.penumbraos.plugins.openai",
            "com.penumbraos.plugins.searxng",
            "com.penumbraos.plugins.system",
        ];

        let rt = tokio::runtime::Handle::current();

        let mut adb = match rt.block_on(AdbManager::connect()) {
            Ok(adb) => adb,
            Err(_) => {
                return Ok(vec![]);
            }
        };

        let mut installed_packages = Vec::new();

        for package_name in package_names {
            // Use dumpsys to check if package exists and get version info
            match rt.block_on(adb.shell(&format!("dumpsys package {}", package_name))) {
                Ok(output)
                    if !output.trim().is_empty() && !output.contains("Unable to find package") =>
                {
                    let version = output
                        .lines()
                        .find(|line| line.trim().starts_with("versionName="))
                        .and_then(|line| line.split("versionName=").nth(1))
                        .map(|version| version.trim().to_string());

                    installed_packages.push(PackageInfo {
                        package_name: package_name.to_string(),
                        version,
                    });
                }
                _ => {
                    // Package not installed, skip
                }
            }
        }

        Ok(installed_packages)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn get_available_repositories() -> Result<Vec<RepositoryInfo>, String> {
    let config = ConfigLoader::load_builtin("penumbra")
        .map_err(|e| format!("Failed to load config: {}", e))?;

    let repos: Vec<RepositoryInfo> = config.repositories.iter().map(|repo| repo.into()).collect();

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

    let selected_repos = if repos.is_empty() {
        "all repositories"
    } else {
        "selected repositories"
    };
    let _ = app.emit(
        "installation_progress",
        format!("Installing {}...", selected_repos),
    );
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let _ = app.emit(
        "installation_progress",
        "Installation completed successfully!",
    );

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
