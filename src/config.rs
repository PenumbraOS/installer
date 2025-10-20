use crate::{InstallerError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstallConfig {
    pub name: String,
    #[serde(default)]
    pub variables: Vec<ConfigVariable>,
    pub repositories: Vec<Repository>,
    #[serde(default)]
    pub global_setup: Vec<InstallStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigVariable {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
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
        for variable in &config.variables {
            if !variable.required && variable.default.is_none() {
                return Err(InstallerError::Config(format!(
                    "Optional variable '{}' must define a default value",
                    variable.name
                )));
            }
        }

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
    pub fn resolve_variables(
        &self,
        overrides: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>> {
        let mut resolved = HashMap::new();

        for key in overrides.keys() {
            if !self.variables.iter().any(|var| &var.name == key) {
                return Err(InstallerError::Config(format!(
                    "Unknown variable override '{}'",
                    key
                )));
            }
        }

        for variable in &self.variables {
            let mut value = variable.default.clone();

            if let Some(override_value) = overrides.get(&variable.name) {
                value = Some(override_value.clone());
            }

            match value {
                Some(v) => {
                    resolved.insert(variable.name.clone(), v);
                }
                None => {
                    if variable.required {
                        return Err(InstallerError::Config(format!(
                            "Missing value for required variable '{}'",
                            variable.name
                        )));
                    }
                }
            }
        }

        Ok(resolved)
    }

    pub fn apply_variables(&mut self, values: &HashMap<String, String>) -> Result<()> {
        substitute_string(&mut self.name, values)?;

        for repo in &mut self.repositories {
            substitute_repository(repo, values)?;
        }

        for step in &mut self.global_setup {
            substitute_install_step(step, values)?;
        }

        Ok(())
    }

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

fn replace_placeholders(input: &str, values: &HashMap<String, String>) -> Result<String> {
    if !input.contains("{{") {
        return Ok(input.to_string());
    }

    let mut output = String::with_capacity(input.len());
    let mut rest = input;
    let mut missing = HashSet::new();

    while let Some(start) = rest.find("{{") {
        output.push_str(&rest[..start]);
        rest = &rest[start + 2..];

        let end = rest.find("}}").ok_or_else(|| {
            InstallerError::Config("Unterminated variable placeholder".to_string())
        })?;

        let key_raw = &rest[..end];
        rest = &rest[end + 2..];

        let key = key_raw.trim();
        if key.is_empty() {
            return Err(InstallerError::Config(
                "Variable placeholder cannot be empty".to_string(),
            ));
        }

        if let Some(value) = values.get(key) {
            output.push_str(value);
        } else {
            missing.insert(key.to_string());
        }
    }

    output.push_str(rest);

    if !missing.is_empty() {
        return Err(InstallerError::Config(format!(
            "Missing values for variables: {}",
            missing.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }

    Ok(output)
}

fn substitute_repository(repo: &mut Repository, values: &HashMap<String, String>) -> Result<()> {
    for cleanup in &mut repo.cleanup {
        substitute_cleanup_step(cleanup, values)?;
    }

    for step in &mut repo.installation {
        substitute_install_step(step, values)?;
    }

    Ok(())
}

fn substitute_cleanup_step(step: &mut CleanupStep, values: &HashMap<String, String>) -> Result<()> {
    match step {
        CleanupStep::UninstallPackages { patterns }
        | CleanupStep::RemoveDirectories { paths: patterns }
        | CleanupStep::RemoveDirectoriesIfEmpty { paths: patterns }
        | CleanupStep::RemoveFiles { paths: patterns } => substitute_strings(patterns, values),
    }
}

fn substitute_install_step(step: &mut InstallStep, values: &HashMap<String, String>) -> Result<()> {
    match step {
        InstallStep::CreateDirectories { paths } => substitute_strings(paths, values),
        InstallStep::InstallApks {
            priority_order,
            exclude_patterns,
            ..
        } => {
            substitute_strings(priority_order, values)?;
            substitute_strings(exclude_patterns, values)
        }
        InstallStep::PushFiles { files } => {
            for file in files {
                substitute_string(&mut file.local, values)?;
                substitute_string(&mut file.remote, values)?;
                if let Some(chmod) = &mut file.chmod {
                    substitute_string(chmod, values)?;
                }
            }
            Ok(())
        }
        InstallStep::GrantPermissions { grants } => {
            for grant in grants {
                substitute_string(&mut grant.package, values)?;
                substitute_string(&mut grant.permission, values)?;
            }
            Ok(())
        }
        InstallStep::SetAppOps { ops } => {
            for op in ops {
                substitute_string(&mut op.package, values)?;
                substitute_string(&mut op.operation, values)?;
                substitute_string(&mut op.mode, values)?;
            }
            Ok(())
        }
        InstallStep::RunCommand { command, .. } => substitute_string(command, values),
        InstallStep::SetLauncher { component } => substitute_string(component, values),
        InstallStep::CreateConfig { path, content, .. } => {
            substitute_string(path, values)?;
            substitute_string(content, values)
        }
    }
}

fn substitute_string(target: &mut String, values: &HashMap<String, String>) -> Result<()> {
    *target = replace_placeholders(target, values)?;
    Ok(())
}

fn substitute_strings(strings: &mut Vec<String>, values: &HashMap<String, String>) -> Result<()> {
    for string in strings {
        substitute_string(string, values)?;
    }
    Ok(())
}
