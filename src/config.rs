use crate::{InstallerError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallConfig {
    pub name: String,
    pub repositories: Vec<Repository>,
    #[serde(default)]
    pub global_setup: Vec<InstallStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    pub name: String,
    pub owner: String,
    pub repo: String,
    pub version: VersionSpec,

    #[serde(default)]
    pub reboot_after_completion: bool,

    #[serde(default)]
    pub cleanup: Vec<CleanupStep>,
    #[serde(rename = "releaseAssets")]
    pub release_assets: Vec<String>,
    #[serde(default, rename = "repoFiles")]
    pub repo_files: Vec<String>,
    pub installation: Vec<InstallStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum VersionSpec {
    Version(String),
}

impl Default for VersionSpec {
    fn default() -> Self {
        VersionSpec::Version("latest".to_string())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum CleanupStep {
    UninstallPackages { patterns: Vec<String> },
    RemoveDirectories { paths: Vec<String> },
    RemoveDirectoriesIfEmpty { paths: Vec<String> },
    RemoveFiles { paths: Vec<String> },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum InstallStep {
    CreateDirectories {
        paths: Vec<String>,
    },
    InstallApks {
        priority_order: Vec<String>,
        #[serde(default)]
        allow_failures: bool,
        #[serde(default)]
        exclude_patterns: Vec<String>,
    },
    PushFiles {
        files: Vec<FilePush>,
    },
    GrantPermissions {
        grants: Vec<PermissionGrant>,
    },
    SetAppOps {
        ops: Vec<AppOpGrant>,
    },
    RunCommand {
        command: String,
        #[serde(default)]
        ignore_failure: bool,
    },
    SetLauncher {
        component: String,
    },
    CreateConfig {
        path: String,
        content: String,
        #[serde(default)]
        only_if_missing: bool,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilePush {
    pub local: String,
    pub remote: String,
    #[serde(default)]
    pub chmod: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PermissionGrant {
    pub package: String,
    pub permission: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppOpGrant {
    pub package: String,
    pub operation: String,
    pub mode: String,
}

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load_builtin(name: &str) -> Result<InstallConfig> {
        match name {
            "penumbra" => {
                let config_str = include_str!("../configs/penumbra.yml");
                Self::load_from_str(config_str)
            }
            _ => Err(InstallerError::Config(format!(
                "Unknown built-in config: {}",
                name
            ))),
        }
    }

    pub async fn load_from_file(path: &Path) -> Result<InstallConfig> {
        let config_str = tokio::fs::read_to_string(path).await?;
        Self::load_from_str(&config_str)
    }

    pub fn load_from_str(config_str: &str) -> Result<InstallConfig> {
        let config: InstallConfig = serde_yaml::from_str(config_str)?;
        Self::validate_config(&config)?;
        Ok(config)
    }

    pub async fn load_from_url(url: &str) -> Result<InstallConfig> {
        let client = reqwest::Client::new();
        let config_str = client.get(url).send().await?.text().await?;
        Self::load_from_str(&config_str)
    }

    fn validate_config(config: &InstallConfig) -> Result<()> {
        if config.repositories.is_empty() {
            return Err(InstallerError::Config(
                "Configuration must have at least one repository".to_string(),
            ));
        }

        let mut names = std::collections::HashSet::new();
        for repo in &config.repositories {
            if !names.insert(&repo.name) {
                return Err(InstallerError::Config(format!(
                    "Duplicate repository name: {}",
                    repo.name
                )));
            }

            if repo.owner.is_empty() || repo.repo.is_empty() {
                return Err(InstallerError::Config(format!(
                    "Repository '{}' must have owner and repo",
                    repo.name
                )));
            }

            if repo.release_assets.is_empty() && repo.repo_files.is_empty() {
                return Err(InstallerError::Config(format!(
                    "Repository '{}' must have at least one release asset or repo file",
                    repo.name
                )));
            }
        }

        Ok(())
    }
}

impl InstallConfig {
    pub fn get_repository(&self, name: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.name == name)
    }

    pub fn filter_repositories(&self, names: &[String]) -> Result<Vec<&Repository>> {
        let mut filtered = Vec::new();

        for name in names {
            if let Some(repo) = self.get_repository(&name) {
                filtered.push(repo);
            } else {
                return Err(InstallerError::RepositoryNotFound { repo: name.clone() });
            }
        }

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = InstallConfig {
            name: "Test".to_string(),
            repositories: vec![],
            global_setup: vec![],
        };

        assert!(ConfigLoader::validate_config(&config).is_err());
    }

    #[test]
    fn test_version_spec_deserialization() {
        let yaml = r#""latest""#;
        let spec: VersionSpec = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(spec, VersionSpec::Version(ref v) if v == "latest"));

        let yaml = r#""2025.09.02.0""#;
        let spec: VersionSpec = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(spec, VersionSpec::Version(ref v) if v == "2025.09.02.0"));
    }
}
