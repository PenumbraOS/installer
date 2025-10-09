pub mod adb;
pub mod config;
pub mod engine;
pub mod error;
pub mod github;
pub mod logs;
pub mod platform;

pub use adb::AdbManager;
pub use config::{ConfigLoader, InstallConfig};
pub use engine::InstallationEngine;
pub use error::{InstallerError, Result};

pub use config::{
    AppOpGrant, CleanupStep, FilePush, InstallStep, PermissionGrant, Repository, VersionSpec,
};
