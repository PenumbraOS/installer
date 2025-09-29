use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::App;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SetupConfig {
    #[serde(default)]
    pub adb_source: Option<AdbSource>,
    #[serde(default)]
    pub github_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdbSource {
    LocalCopy {
        stored_path: String,
        original_filename: String,
    },
    RemoteServer {
        url: String,
    },
}

pub struct SetupState {
    config_path: PathBuf,
    files_dir: PathBuf,
    inner: Mutex<SetupConfig>,
}

#[derive(Debug)]
pub struct SetupStateError(pub String);

impl std::fmt::Display for SetupStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SetupStateError {}

impl SetupState {
    pub fn initialize(app: &App) -> Result<Self, SetupStateError> {
        let config_dir = app
            .path()
            .app_config_dir()
            .or_else(|_| {
                Err(SetupStateError(
                    "Failed to resolve app config directory".to_string(),
                ))
            })?
            .join("penumbra-installer");

        fs::create_dir_all(&config_dir)
            .map_err(|e| SetupStateError(format!("Failed to create config directory: {}", e)))?;

        let files_dir = config_dir.join("adb_keys");
        fs::create_dir_all(&files_dir)
            .map_err(|e| SetupStateError(format!("Failed to create key directory: {}", e)))?;

        let config_path = config_dir.join("setup_state.json");
        let config = Self::load_from_disk(&config_path)?;

        Ok(Self {
            config_path,
            files_dir,
            inner: Mutex::new(config),
        })
    }

    pub fn get_config(&self) -> Result<SetupConfig, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "Setup state poisoned".to_string())?;
        Ok(guard.clone())
    }

    pub fn set_remote_server(&self, url: String) -> Result<SetupConfig, String> {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Err("Remote signing server URL cannot be empty".to_string());
        }

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "Setup state poisoned".to_string())?;

        guard.adb_source = Some(AdbSource::RemoteServer {
            url: trimmed.to_string(),
        });

        self.persist(&guard)?;
        Ok(guard.clone())
    }

    pub fn clear_adb_source(&self) -> Result<SetupConfig, String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "Setup state poisoned".to_string())?;

        if let Some(AdbSource::LocalCopy { stored_path, .. }) = &guard.adb_source {
            if let Err(e) = fs::remove_file(stored_path) {
                if e.kind() != io::ErrorKind::NotFound {
                    eprintln!("Failed to remove stored ADB key: {}", e);
                }
            }
        }

        guard.adb_source = None;
        self.persist(&guard)?;
        Ok(guard.clone())
    }

    pub fn set_local_file(&self, source: PathBuf) -> Result<SetupConfig, String> {
        if !source.exists() {
            return Err(format!("Key file does not exist: {}", source.display()));
        }

        let metadata = source
            .metadata()
            .map_err(|e| format!("Failed to read key file metadata: {}", e))?;

        if !metadata.is_file() {
            return Err("Provided path is not a file".to_string());
        }

        fs::create_dir_all(&self.files_dir)
            .map_err(|e| format!("Failed to prepare storage directory: {}", e))?;

        let original_filename = source
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "adb_key".to_string());

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();

        let mut dest_name = format!("adb_key_{}_{}", timestamp, original_filename);
        dest_name = dest_name.replace(['\\', '/', ':'], "_");

        let dest_path = self.files_dir.join(dest_name);

        fs::copy(&source, &dest_path).map_err(|e| format!("Failed to copy key file: {}", e))?;

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "Setup state poisoned".to_string())?;

        if let Some(AdbSource::LocalCopy { stored_path, .. }) = &guard.adb_source {
            if *stored_path != dest_path.to_string_lossy() {
                if let Err(e) = fs::remove_file(stored_path) {
                    if e.kind() != io::ErrorKind::NotFound {
                        eprintln!("Failed to remove previous ADB key copy: {}", e);
                    }
                }
            }
        }

        guard.adb_source = Some(AdbSource::LocalCopy {
            stored_path: dest_path.to_string_lossy().to_string(),
            original_filename,
        });

        self.persist(&guard)?;
        Ok(guard.clone())
    }

    pub fn set_local_bytes(
        &self,
        original_filename: String,
        data: Vec<u8>,
    ) -> Result<SetupConfig, String> {
        if data.is_empty() {
            return Err("Key file is empty".to_string());
        }

        fs::create_dir_all(&self.files_dir)
            .map_err(|e| format!("Failed to prepare storage directory: {}", e))?;

        let sanitized_name = if original_filename.trim().is_empty() {
            "adb_key".to_string()
        } else {
            original_filename.trim().to_string()
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();

        let mut dest_name = format!("adb_key_{}_{}", timestamp, sanitized_name);
        dest_name = dest_name.replace(['\\', '/', ':'], "_");

        let dest_path = self.files_dir.join(dest_name);

        fs::write(&dest_path, &data).map_err(|e| format!("Failed to write key file: {}", e))?;

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "Setup state poisoned".to_string())?;

        if let Some(AdbSource::LocalCopy { stored_path, .. }) = &guard.adb_source {
            if *stored_path != dest_path.to_string_lossy() {
                if let Err(e) = fs::remove_file(stored_path) {
                    if e.kind() != io::ErrorKind::NotFound {
                        eprintln!("Failed to remove previous ADB key copy: {}", e);
                    }
                }
            }
        }

        guard.adb_source = Some(AdbSource::LocalCopy {
            stored_path: dest_path.to_string_lossy().to_string(),
            original_filename: sanitized_name,
        });

        self.persist(&guard)?;
        Ok(guard.clone())
    }

    pub fn set_github_token(&self, token: Option<String>) -> Result<SetupConfig, String> {
        let normalized = token.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "Setup state poisoned".to_string())?;
        guard.github_token = normalized;
        self.persist(&guard)?;
        Ok(guard.clone())
    }

    fn persist(&self, config: &SetupConfig) -> Result<(), String> {
        let tmp_path = self.config_path.with_extension("tmp");
        let data = serde_json::to_vec_pretty(config)
            .map_err(|e| format!("Failed to serialize setup config: {}", e))?;
        fs::write(&tmp_path, data).map_err(|e| format!("Failed to write setup config: {}", e))?;
        fs::rename(&tmp_path, &self.config_path)
            .map_err(|e| format!("Failed to persist setup config: {}", e))?;
        Ok(())
    }

    fn load_from_disk(path: &Path) -> Result<SetupConfig, SetupStateError> {
        if path.exists() {
            let file = fs::File::open(path)
                .map_err(|e| SetupStateError(format!("Failed to open setup config: {}", e)))?;
            serde_json::from_reader(file)
                .map_err(|e| SetupStateError(format!("Failed to decode setup config: {}", e)))
        } else {
            Ok(SetupConfig::default())
        }
    }
}
