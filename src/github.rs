use log::{info, warn};
use reqwest::{Client, Response};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::config::VersionSpec;
use crate::platform::Platform;
use crate::{InstallerError, Repository, Result};

pub struct GitHubClient {
    client: Client,
    auth_header: Option<String>,
}

impl GitHubClient {
    pub fn new() -> Self {
        Self::new_with_token(None)
    }

    pub fn new_with_token(token: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent(Platform::user_agent())
            .build()
            .unwrap();

        let auth_header = token.map(|t| format!("Bearer {}", t));

        Self {
            client,
            auth_header,
        }
    }

    pub async fn get_version(&self, repo: &Repository) -> Result<String> {
        match &repo.version {
            VersionSpec::Version(v) if v == "latest" => {
                self.get_latest_version(&repo.owner, &repo.repo).await
            }
            VersionSpec::Version(v) => Ok(v.clone()),
        }
    }

    async fn get_latest_version(&self, owner: &str, repo: &str) -> Result<String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );
        let mut request = self.client.get(&url);

        if let Some(ref auth) = self.auth_header {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await?;

        if response.status().is_success() {
            let json: Value = response.json().await?;
            if let Some(tag_name) = json["tag_name"].as_str() {
                return Ok(tag_name.to_string());
            }
        }

        let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);
        let mut request = self.client.get(&url);

        if let Some(ref auth) = self.auth_header {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await?;

        let json = validate_response(
            response,
            &format!("fetch '{repo}' releases"),
            self.auth_header.is_some(),
        )
        .await?;

        let releases = json
            .as_array()
            .ok_or_else(|| InstallerError::GitHub("Expected array of releases".to_string()))?;

        if releases.is_empty() {
            return Err(InstallerError::GitHub("No releases found".to_string()));
        }

        let latest_release = &releases[0];
        let tag_name = latest_release["tag_name"]
            .as_str()
            .ok_or_else(|| InstallerError::GitHub("No tag_name found in release".to_string()))?;

        Ok(tag_name.to_string())
    }

    pub async fn download_asset(
        &self,
        owner: &str,
        repo: &str,
        version: &str,
        pattern: &str,
        dest_dir: &Path,
        exclude_patterns: &[String],
    ) -> Result<Vec<PathBuf>> {
        fs::create_dir_all(dest_dir).await?;

        let assets = self.get_release_assets(owner, repo, version).await?;
        let mut downloaded_files = Vec::new();

        for asset in assets {
            let name = asset["name"]
                .as_str()
                .ok_or_else(|| InstallerError::GitHub("Asset has no name".to_string()))?;

            if self.matches_pattern(name, pattern) {
                let should_exclude = exclude_patterns
                    .iter()
                    .any(|exclude_pattern| self.matches_pattern(name, exclude_pattern));

                if should_exclude {
                    info!("  Skipping excluded asset: {}", name);
                    continue;
                }

                let download_url = asset["browser_download_url"].as_str().ok_or_else(|| {
                    InstallerError::GitHub("Asset has no download URL".to_string())
                })?;

                let dest_path = dest_dir.join(name);
                self.download_file_from_url(download_url, &dest_path)
                    .await?;
                downloaded_files.push(dest_path);

                info!("  Downloaded: {}", name);
            }
        }

        if downloaded_files.is_empty() {
            warn!("  No assets found matching pattern: {}", pattern);
        }

        Ok(downloaded_files)
    }

    pub async fn download_file(
        &self,
        owner: &str,
        repo: &str,
        version: &str,
        filepath: &str,
        dest: &Path,
    ) -> Result<()> {
        if filepath.contains('*') {
            return self
                .download_files_glob(owner, repo, version, filepath, dest)
                .await;
        }

        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner, repo, version, filepath
        );

        self.download_file_from_url(&url, dest).await
    }

    async fn download_files_glob(
        &self,
        owner: &str,
        repo: &str,
        version: &str,
        filepath: &str,
        dest_dir: &Path,
    ) -> Result<()> {
        let base_path = filepath.split('*').next().unwrap_or("");
        let pattern = filepath.split('/').last().unwrap_or("*");

        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner,
            repo,
            base_path.trim_end_matches('/')
        );

        let mut request = self.client.get(&url);

        if let Some(ref auth) = self.auth_header {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await?;
        let json = validate_response(
            response,
            &format!("list contents of '{repo}'"),
            self.auth_header.is_some(),
        )
        .await?;

        let files = json
            .as_array()
            .ok_or_else(|| InstallerError::GitHub("Expected array of files".to_string()))?;

        fs::create_dir_all(dest_dir).await?;

        for file in files {
            let name = file["name"].as_str().unwrap_or("");
            if self.matches_pattern(name, pattern) {
                let file_url = format!(
                    "https://raw.githubusercontent.com/{}/{}/{}/{}/{}",
                    owner,
                    repo,
                    version,
                    base_path.trim_end_matches('/'),
                    name
                );
                let dest_path = dest_dir.join(name);
                self.download_file_from_url(&file_url, &dest_path).await?;
                info!("  Downloaded: {}", name);
            }
        }

        Ok(())
    }

    async fn get_release_assets(
        &self,
        owner: &str,
        repo: &str,
        version: &str,
    ) -> Result<Vec<Value>> {
        let url = if version == "latest" {
            format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                owner, repo
            )
        } else {
            format!(
                "https://api.github.com/repos/{}/{}/releases/tags/{}",
                owner, repo, version
            )
        };

        let mut request = self.client.get(&url);

        if let Some(ref auth) = self.auth_header {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await?;
        let json = validate_response(
            response,
            &format!("fetch '{repo}'"),
            self.auth_header.is_some(),
        )
        .await?;

        let assets = json["assets"]
            .as_array()
            .ok_or_else(|| InstallerError::GitHub("No assets found in release".to_string()))?;

        Ok(assets.clone())
    }

    async fn download_file_from_url(&self, url: &str, dest: &Path) -> Result<()> {
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(InstallerError::GitHub(format!(
                "Failed to download file: HTTP {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await?;

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(dest, bytes).await?;
        Ok(())
    }

    fn matches_pattern(&self, filename: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();

            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];

                if prefix.is_empty() && suffix.is_empty() {
                    return true;
                } else if prefix.is_empty() {
                    return filename.ends_with(suffix);
                } else if suffix.is_empty() {
                    return filename.starts_with(prefix);
                } else {
                    return filename.starts_with(prefix) && filename.ends_with(suffix);
                }
            } else if parts.len() == 3 && parts[0].is_empty() && parts[2].is_empty() {
                return filename.contains(parts[1]);
            }
        }

        filename == pattern
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

async fn validate_response(response: Response, action: &str, has_auth: bool) -> Result<Value> {
    if !response.status().is_success() {
        let auth_message = if has_auth {
            "using auth"
        } else {
            "without auth"
        };

        let status_code = response.status();

        let json: std::result::Result<Value, reqwest::Error> = response.json().await;

        Err(InstallerError::GitHub(format!(
            "Failed to {action} {auth_message}: HTTP {status_code}, body: {json:?}",
        )))
    } else {
        let json: Value = response.json().await?;

        Ok(json)
    }
}
