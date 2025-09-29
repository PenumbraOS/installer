#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod setup;

use log::{warn, Level, Metadata, Record};
use once_cell::sync::Lazy;
use penumbra_installer::{
    AdbManager, ConfigLoader, InstallConfig, InstallationEngine, InstallerError, Repository,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::{runtime::Handle, task::spawn_blocking};
use tokio_util::sync::CancellationToken;

struct TauriLogger {
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl TauriLogger {
    fn new() -> Self {
        Self {
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    fn set_app_handle(&self, app: AppHandle) {
        let mut handle = self.app_handle.lock().unwrap();
        *handle = Some(app);
    }
}

impl log::Log for TauriLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{}", record.args());

            println!("{message}");

            if let Ok(handle_guard) = self.app_handle.lock() {
                if let Some(ref app) = *handle_guard {
                    let _ = app.emit("installation_progress", &message);
                }
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: Lazy<TauriLogger> = Lazy::new(|| TauriLogger::new());

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
    cancellation_token: Mutex<Option<CancellationToken>>,
}

#[tauri::command]
fn load_setup_config(state: State<'_, setup::SetupState>) -> Result<setup::SetupConfig, String> {
    state.get_config()
}

#[tauri::command]
fn set_adb_key_from_file(
    path: String,
    state: State<'_, setup::SetupState>,
) -> Result<setup::SetupConfig, String> {
    state.set_local_file(PathBuf::from(path))
}

#[tauri::command]
fn set_adb_key_remote(
    url: String,
    state: State<'_, setup::SetupState>,
) -> Result<setup::SetupConfig, String> {
    state.set_remote_server(url)
}

#[tauri::command]
fn set_adb_key_from_bytes(
    filename: String,
    data: Vec<u8>,
    state: State<'_, setup::SetupState>,
) -> Result<setup::SetupConfig, String> {
    state.set_local_bytes(filename, data)
}

#[tauri::command]
fn clear_adb_key(state: State<'_, setup::SetupState>) -> Result<setup::SetupConfig, String> {
    state.clear_adb_source()
}

#[tauri::command]
fn set_github_token(
    token: Option<String>,
    state: State<'_, setup::SetupState>,
) -> Result<setup::SetupConfig, String> {
    state.set_github_token(token)
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
    let _ = app.emit("installation_progress", "Loading configuration...");

    let config = ConfigLoader::load_builtin("penumbra").map_err(|e| {
        let _ = app.emit(
            "installation_progress",
            format!("Error: Failed to load config - {}", e),
        );
        format!("Failed to load config: {}", e)
    })?;

    let _ = app.emit("installation_progress", "Starting installation...");

    let cancellation_token = CancellationToken::new();

    {
        let mut token = state.cancellation_token.lock().unwrap();
        *token = Some(cancellation_token.clone());
    }

    let installation_result = run_installation(config, repos, cancellation_token.clone()).await;

    {
        let mut token = state.cancellation_token.lock().unwrap();
        *token = None;
    }

    match installation_result {
        Ok(()) => {
            let _ = app.emit(
                "installation_progress",
                "Installation completed successfully!",
            );
            Ok("Installation completed successfully".to_string())
        }
        Err(error_msg) => {
            let _ = app.emit("installation_progress", format!("Error: {}", error_msg));
            Err(error_msg)
        }
    }
}

async fn run_installation(
    config: InstallConfig,
    repos: Vec<String>,
    cancellation_token: CancellationToken,
) -> Result<(), String> {
    spawn_blocking(move || {
        let rt = Handle::current();

        let mut engine = match rt.block_on(InstallationEngine::new_with_token(
            config,
            None,
            Some(cancellation_token),
        )) {
            Ok(engine) => engine,
            Err(e) => return Err(format!("Failed to initialize installation engine: {}", e)),
        };

        let repo_filter = if repos.is_empty() { None } else { Some(repos) };

        match rt.block_on(engine.install(repo_filter, false)) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Installation failed: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn cancel_installation(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut token = state.cancellation_token.lock().unwrap();
        if let Some(cancellation_token) = token.take() {
            cancellation_token.cancel();
            warn!("Cancelled installation");
        }
    }

    Ok(())
}

fn main() {
    log::set_logger(&*LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .expect("Failed to initialize logger");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            cancellation_token: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            load_setup_config,
            set_adb_key_from_file,
            set_adb_key_from_bytes,
            set_adb_key_remote,
            clear_adb_key,
            set_github_token,
            check_device_connection,
            list_installed_packages,
            get_available_repositories,
            install_repositories,
            cancel_installation
        ])
        .setup(|app| {
            LOGGER.set_app_handle(app.handle().clone());

            let setup_state = setup::SetupState::initialize(app)
                .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
            app.manage(setup_state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
