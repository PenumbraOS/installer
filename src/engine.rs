use glob::glob;
use log::{info, warn};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::adb::AdbManager;
use crate::github::GitHubClient;
use crate::platform::Platform;
use crate::{
    CleanupStep, FilePush, InstallConfig, InstallStep, InstallerError, Repository, Result,
};

pub struct InstallationEngine {
    config: InstallConfig,
    github: GitHubClient,
    adb: AdbManager,
    temp_dir: PathBuf,
    cancellation_token: Option<CancellationToken>,
}

impl InstallationEngine {
    pub async fn new(config: InstallConfig) -> Result<Self> {
        InstallationEngine::new_with_cache(config, Platform::temp_dir(), None, None).await
    }

    pub async fn new_with_token(
        config: InstallConfig,
        github_token: Option<String>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<Self> {
        InstallationEngine::new_with_cache(
            config,
            Platform::temp_dir(),
            github_token,
            cancellation_token,
        )
        .await
    }

    pub async fn new_with_cache(
        config: InstallConfig,
        cache_dir: PathBuf,
        github_token: Option<String>,
        cancellation_token: Option<CancellationToken>,
    ) -> Result<Self> {
        fs::create_dir_all(&cache_dir).await?;

        let github = GitHubClient::new_with_token(github_token);
        let adb = AdbManager::connect().await?;

        Ok(Self {
            config,
            github,
            adb,
            temp_dir: cache_dir,
            cancellation_token,
        })
    }

    pub async fn install(
        &mut self,
        repo_filter: Option<Vec<String>>,
        with_cache: bool,
    ) -> Result<()> {
        info!("Starting {} installation", self.config.name);

        if !self.config.global_setup.is_empty() {
            info!("Running global setup");
            let global_setup = self.config.global_setup.clone();
            for step in &global_setup {
                self.execute_install_step(step, "global").await?;
            }
        }

        let repos_to_install: Vec<_> = if let Some(filter) = repo_filter {
            self.config
                .filter_repositories(&filter)?
                .into_iter()
                .cloned()
                .collect()
        } else {
            self.config.repositories.clone()
        };

        if repos_to_install.is_empty() {
            return Err(InstallerError::NoRepositoriesFound);
        }

        info!("Installing {} repositories", repos_to_install.len());

        for repo in &repos_to_install {
            if self.is_cancelled() {
                break;
            }

            info!("Installing repository: {}", repo.name);
            self.install_repository(repo, with_cache).await?;
        }

        if !with_cache {
            info!("Cleaning up temporary files");
            fs::remove_dir_all(&self.temp_dir).await?;
        }

        info!("Installation complete");

        if !self.is_cancelled()
            && self
                .config
                .repositories
                .iter()
                .any(|r| r.reboot_after_completion)
        {
            info!("Rebooting device");
            self.adb.reboot()?;
        }

        Ok(())
    }

    pub async fn uninstall(&mut self, repo_filter: Option<Vec<String>>) -> Result<()> {
        info!("Starting {} uninstall", self.config.name);

        let repos_to_uninstall: Vec<_> = if let Some(filter) = repo_filter {
            self.config
                .filter_repositories(&filter)?
                .into_iter()
                .cloned()
                .collect()
        } else {
            self.config.repositories.clone()
        };

        if repos_to_uninstall.is_empty() {
            return Err(InstallerError::NoRepositoriesFound);
        }

        info!("Uninstalling {} repositories", repos_to_uninstall.len());

        for repo in repos_to_uninstall.iter().rev() {
            info!("Uninstalling repository: {}", repo.name);
            self.uninstall_repository(repo).await?;
        }

        info!("Uninstallation complete");
        Ok(())
    }

    pub async fn download(&mut self, repo_filter: Option<Vec<String>>) -> Result<()> {
        info!("Starting {} asset download", self.config.name);

        let repos_to_download: Vec<_> = if let Some(filter) = repo_filter {
            self.config
                .filter_repositories(&filter)?
                .into_iter()
                .cloned()
                .collect()
        } else {
            self.config.repositories.clone()
        };

        if repos_to_download.is_empty() {
            return Err(InstallerError::NoRepositoriesFound);
        }

        info!("Downloading {} repositories", repos_to_download.len());

        for repo in &repos_to_download {
            info!("Downloading repository: {}", repo.name);
            self.download_repository(repo).await?;
        }

        info!("Download complete - assets cached for installation");
        Ok(())
    }

    async fn install_repository(&mut self, repo: &Repository, with_cache: bool) -> Result<()> {
        if with_cache {
            let repo_temp_dir = self.temp_dir.join(&repo.name);

            if !repo_temp_dir.exists() {
                return Err(InstallerError::Config(format!(
                    "No cached assets found for repository '{}'. Run 'penumbra download' first.",
                    repo.name
                )));
            }
        } else {
            self.download_repository_assets(repo).await?;
        }

        if !repo.cleanup.is_empty() {
            info!("Running cleanup for {}", repo.name);
            for cleanup in &repo.cleanup {
                if self.is_cancelled() {
                    break;
                }

                self.execute_cleanup_step(cleanup).await?;
            }
        }

        info!("Running installation steps for {}", repo.name);
        for step in &repo.installation {
            if self.is_cancelled() {
                break;
            }

            self.execute_install_step(step, &repo.name).await?;
        }

        info!("{} installation complete", repo.name);
        Ok(())
    }

    async fn uninstall_repository(&mut self, repo: &Repository) -> Result<()> {
        if repo.cleanup.is_empty() {
            info!("No cleanup steps defined for {}", repo.name);
            return Ok(());
        }

        info!("Running cleanup steps for {}", repo.name);
        for cleanup in &repo.cleanup {
            self.execute_cleanup_step(cleanup).await?;
        }

        info!("{} uninstallation complete", repo.name);
        Ok(())
    }

    async fn download_repository(&mut self, repo: &Repository) -> Result<()> {
        self.download_repository_assets(repo).await?;
        info!("{} download complete", repo.name);
        Ok(())
    }

    async fn execute_cleanup_step(&mut self, step: &CleanupStep) -> Result<()> {
        match step {
            CleanupStep::UninstallPackages { patterns } => {
                for pattern in patterns {
                    let packages = self.find_packages_matching_pattern(pattern).await?;
                    for package in packages {
                        info!("Uninstalling package: {}", package);
                        self.adb.uninstall_package(&package).await?;
                    }
                }
            }
            CleanupStep::RemoveDirectories { paths } => {
                for path in paths {
                    info!("Removing directory: {}", path);
                    self.adb.remove_directory(path).await?;
                }
            }
            CleanupStep::RemoveDirectoriesIfEmpty { paths } => {
                for path in paths {
                    if self.is_directory_empty(path).await? {
                        info!("Removing empty directory: {}", path);
                        self.adb.remove_directory(path).await?;
                    } else {
                        warn!("Directory not empty, skipping: {}", path);
                    }
                }
            }
            CleanupStep::RemoveFiles { paths } => {
                for path in paths {
                    info!("Removing file: {}", path);
                    self.adb.remove_file(path).await?;
                }
            }
        }
        Ok(())
    }

    async fn execute_install_step(&mut self, step: &InstallStep, repo_name: &str) -> Result<()> {
        match step {
            InstallStep::CreateDirectories { paths } => {
                for path in paths {
                    info!("Creating directory: {}", path);
                    self.adb.create_directory(path).await?;
                }
            }

            InstallStep::InstallApks {
                priority_order,
                allow_failures,
                exclude_patterns,
            } => {
                let repo_temp_dir = if repo_name == "global" {
                    self.temp_dir.clone()
                } else {
                    self.temp_dir.join(repo_name)
                };

                let mut apks = self.find_apk_files_in_dir(&repo_temp_dir)?;

                apks.retain(|apk| {
                    let filename = apk.file_name().unwrap().to_string_lossy();
                    !exclude_patterns
                        .iter()
                        .any(|pattern| self.matches_priority_pattern(&filename, pattern))
                });

                if apks.is_empty() {
                    info!("No APK files found to install");
                    return Ok(());
                }

                let sorted_apks = self.sort_apks_by_priority(&apks, priority_order);

                info!("Installing {} APKs", sorted_apks.len());
                for apk in sorted_apks {
                    if self.is_cancelled() {
                        break;
                    }

                    let apk_name = apk.file_name().unwrap().to_string_lossy();
                    info!("Installing APK: {}", apk_name);

                    match self.adb.install_apk(&apk).await {
                        Ok(()) => info!("Installed APK: {}", apk_name),
                        Err(e) if *allow_failures => {
                            warn!("Failed to install {} (continuing): {}", apk_name, e);
                        }
                        Err(e) => return Err(e),
                    }
                }
            }

            InstallStep::PushFiles { files } => {
                let repo_temp_dir = if repo_name == "global" {
                    self.temp_dir.clone()
                } else {
                    self.temp_dir.join(repo_name)
                };

                for file_push in files {
                    if self.is_cancelled() {
                        break;
                    }

                    self.push_files(&repo_temp_dir, file_push).await?;
                }
            }

            InstallStep::GrantPermissions { grants } => {
                for grant in grants {
                    info!(
                        "Granting permission: {} to {}",
                        grant.permission, grant.package
                    );
                    self.adb
                        .grant_permission(&grant.package, &grant.permission)
                        .await?;
                }
            }

            InstallStep::SetAppOps { ops } => {
                for i in 0..3 {
                    if i != 0 {
                        info!("Delaying 5s to ensure app op changes succeed");
                        sleep(Duration::from_secs(5)).await;
                    }

                    for op in ops {
                        if self.is_cancelled() {
                            break;
                        }

                        info!(
                            "Setting app op: {} {} {}",
                            op.package, op.operation, op.mode
                        );
                        self.adb
                            .set_app_op(&op.package, &op.operation, &op.mode)
                            .await?;
                    }
                }
            }

            InstallStep::RunCommand {
                command,
                ignore_failure,
            } => {
                info!("Running command: {}", command);
                match self.adb.shell(command).await {
                    Ok(output) => {
                        if !output.is_empty() {
                            info!("Command output: {}", output);
                        }
                    }
                    Err(e) if *ignore_failure => {
                        warn!("Command failed (ignoring): {}", e);
                    }
                    Err(e) => return Err(e),
                }
            }

            InstallStep::SetLauncher { component } => {
                info!("Setting launcher: {}", component);
                self.adb.set_launcher(component).await?;
            }

            InstallStep::CreateConfig {
                path,
                content,
                only_if_missing,
            } => {
                if *only_if_missing && self.adb.file_exists(path).await? {
                    info!("Config already exists: {}", path);
                    return Ok(());
                }

                info!("Creating config: {}", path);
                self.adb.write_file(path, content).await?;
            }
        }
        Ok(())
    }

    async fn push_files(&mut self, repo_temp_dir: &Path, file_push: &FilePush) -> Result<()> {
        let local_pattern = repo_temp_dir.join(&file_push.local);
        let pattern_str = local_pattern.to_string_lossy();

        for entry in glob(&pattern_str)? {
            let local_file = entry?;

            let remote_path = if file_push.remote.ends_with('/') {
                format!(
                    "{}{}",
                    file_push.remote,
                    local_file.file_name().unwrap().to_string_lossy()
                )
            } else {
                file_push.remote.clone()
            };

            info!(
                "Pushing: {} -> {}",
                local_file.file_name().unwrap().to_string_lossy(),
                remote_path
            );

            self.adb.push_file(&local_file, &remote_path).await?;

            if let Some(chmod) = &file_push.chmod {
                self.adb
                    .shell(&format!("chmod {} {}", chmod, remote_path))
                    .await?;
            }
        }

        Ok(())
    }

    fn find_apk_files_in_dir(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut apks = Vec::new();
        let pattern = dir.join("*.apk");

        for entry in glob(&pattern.to_string_lossy())? {
            let path = entry?;
            if path.is_file() {
                apks.push(path);
            }
        }

        Ok(apks)
    }

    fn sort_apks_by_priority(&self, apks: &[PathBuf], priority_order: &[String]) -> Vec<PathBuf> {
        let mut sorted_apks = Vec::new();
        let mut remaining_apks = apks.to_vec();

        for priority_pattern in priority_order {
            let mut matched_apks = Vec::new();

            remaining_apks.retain(|apk| {
                let filename = apk.file_name().unwrap().to_string_lossy();
                let matches = self.matches_priority_pattern(&filename, priority_pattern);
                if matches {
                    matched_apks.push(apk.clone());
                }
                !matches
            });

            sorted_apks.extend(matched_apks);
        }

        sorted_apks.extend(remaining_apks);

        sorted_apks
    }

    fn matches_priority_pattern(&self, filename: &str, pattern: &str) -> bool {
        if pattern.starts_with('*') && pattern.ends_with('*') {
            let inner = &pattern[1..pattern.len() - 1];
            filename.to_lowercase().contains(&inner.to_lowercase())
        } else if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            filename.to_lowercase().ends_with(&suffix.to_lowercase())
        } else if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            filename.to_lowercase().starts_with(&prefix.to_lowercase())
        } else {
            filename.eq_ignore_ascii_case(pattern)
        }
    }

    async fn find_packages_matching_pattern(&mut self, pattern: &str) -> Result<Vec<String>> {
        let search_pattern = if pattern.contains('*') {
            pattern.replace('*', "")
        } else {
            pattern.to_string()
        };

        self.adb.list_packages(&search_pattern).await
    }

    async fn is_directory_empty(&mut self, path: &str) -> Result<bool> {
        let output = self.adb.shell(&format!("ls -A {}", path)).await?;
        Ok(output.trim().is_empty())
    }

    fn get_exclusion_patterns(&self, repo: &Repository) -> Vec<String> {
        for step in &repo.installation {
            if let InstallStep::InstallApks {
                exclude_patterns, ..
            } = step
            {
                return exclude_patterns.clone();
            }
        }
        Vec::new()
    }

    async fn download_repository_assets(&mut self, repo: &Repository) -> Result<()> {
        let version = self.github.get_version(repo).await?;
        info!("Version: {}", version);

        let repo_temp_dir = self.temp_dir.join(&repo.name);
        fs::create_dir_all(&repo_temp_dir).await?;

        let exclude_patterns = self.get_exclusion_patterns(repo);

        info!("Downloading release assets");
        for pattern in &repo.release_assets {
            if self.is_cancelled() {
                break;
            }

            let downloaded = self
                .github
                .download_asset(
                    &repo.owner,
                    &repo.repo,
                    &version,
                    pattern,
                    &repo_temp_dir,
                    &exclude_patterns,
                )
                .await?;

            if downloaded.is_empty() {
                warn!("No release assets found for pattern: {}", pattern);
            }
        }

        for filepath in &repo.repo_files {
            if self.is_cancelled() {
                break;
            }

            info!("Downloading repository file: {}", filepath);
            if filepath.contains('*') {
                self.github
                    .download_file(&repo.owner, &repo.repo, &version, filepath, &repo_temp_dir)
                    .await?;
            } else {
                let dest = repo_temp_dir.join(Path::new(filepath).file_name().unwrap());
                self.github
                    .download_file(&repo.owner, &repo.repo, &version, filepath, &dest)
                    .await?;
            }
        }

        Ok(())
    }

    fn is_cancelled(&self) -> bool {
        self.cancellation_token
            .as_ref()
            .map_or(false, |token| token.is_cancelled())
    }
}
