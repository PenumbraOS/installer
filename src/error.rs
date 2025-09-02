use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallerError {
    #[error("ADB error: {0}")]
    Adb(String),

    #[error("GitHub API error: {0}")]
    GitHub(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Glob pattern error: {0}")]
    Glob(#[from] glob::PatternError),

    #[error("Glob matching error: {0}")]
    GlobMatch(#[from] glob::GlobError),

    #[error("No Android device connected")]
    NoDevice,

    #[error("Multiple devices connected (exactly one required)")]
    MultipleDevices,

    #[error("No repositories found matching filter")]
    NoRepositoriesFound,

    #[error("Repository '{repo}' not found in configuration")]
    RepositoryNotFound { repo: String },

    #[error("Installation step failed: {step}, reason: {reason}")]
    InstallationStep { step: String, reason: String },

    #[error("APK installation failed: {apk}, reason: {reason}")]
    ApkInstallation { apk: String, reason: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid version format: {version}")]
    InvalidVersion { version: String },
}

pub type Result<T> = std::result::Result<T, InstallerError>;
