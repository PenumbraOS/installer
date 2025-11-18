use clap::{Parser, Subcommand};
use log::{error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio;

use penumbra_installer::{
    logs::dump_logcat_and_exit, ConfigLoader, InstallationEngine, InstallerError, Result,
};

#[derive(Parser)]
#[command(name = "penumbra")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "PenumbraOS official installer")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[arg(long, global = true, env)]
    github_token: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        config_url: Option<String>,
        /// URL for remote ADB authentication
        #[clap(short = 'a', long = "remote-auth-url")]
        remote_auth_url: Option<String>,

        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        variables: Vec<String>,
    },
    Uninstall {
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
        /// URL for remote ADB authentication
        #[clap(short = 'a', long = "remote-auth-url")]
        remote_auth_url: Option<String>,
    },
    Download {
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
        #[arg(long)]
        cache_dir: PathBuf,
    },
    List {
        config: Option<PathBuf>,
    },
    Devices {
        /// URL for remote ADB authentication
        #[clap(short = 'a', long = "remote-auth-url")]
        remote_auth_url: Option<String>,
    },
    DumpLogs {
        #[clap(short = 's', long = "stream")]
        stream: bool,

        /// URL for remote ADB authentication
        #[clap(short = 'a', long = "remote-auth-url")]
        remote_auth_url: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    match cli.command {
        Commands::Install {
            repos,
            cache_dir,
            config,
            config_url,
            remote_auth_url,
            variables,
        } => {
            let variable_overrides = parse_variable_overrides(&variables)?;

            let mut config = match (config, config_url) {
                (None, None) => ConfigLoader::load_builtin("penumbra"),
                (None, Some(config_url)) => ConfigLoader::load_from_url(&config_url).await,
                (Some(config_path), None) => ConfigLoader::load_from_file(&config_path).await,
                (Some(_), Some(_)) => {
                    return Err(InstallerError::CLI(
                        "`config` and `config_url` options are mutually exclusive".into(),
                    ));
                }
            }?;

            let mut active_repos = config.filter_repositories(repos)?;
            config.resolve_and_apply_variables(&mut active_repos, &variable_overrides)?;

            let mut engine = if let Some(ref cache_path) = cache_dir {
                InstallationEngine::new_with_cache(
                    config,
                    cache_path.clone(),
                    cli.github_token.clone(),
                    remote_auth_url.clone(),
                    None,
                )
                .await?
            } else {
                InstallationEngine::new_with_token(
                    config,
                    cli.github_token.clone(),
                    remote_auth_url.clone(),
                    None,
                )
                .await?
            };

            engine.install(&active_repos, cache_dir.is_some()).await?;
        }

        Commands::Uninstall {
            repos,
            remote_auth_url,
        } => {
            let config = ConfigLoader::load_builtin("penumbra")?;
            let mut engine = InstallationEngine::new_with_token(
                config,
                cli.github_token.clone(),
                remote_auth_url,
                None,
            )
            .await?;
            let active_repos = engine.config.filter_repositories(repos)?;
            engine.uninstall(&active_repos).await?;
        }

        Commands::Download { repos, cache_dir } => {
            let config = ConfigLoader::load_builtin("penumbra")?;
            let mut engine = InstallationEngine::new_with_cache(
                config,
                cache_dir,
                cli.github_token.clone(),
                None,
                None,
            )
            .await?;
            let active_repos = engine.config.filter_repositories(repos)?;
            engine.download(&active_repos).await?;
        }

        Commands::List { config } => {
            let config = if let Some(config_path) = config {
                ConfigLoader::load_from_file(&config_path).await?
            } else {
                ConfigLoader::load_builtin("penumbra")?
            };

            info!("Available repositories in '{}':", config.name);
            for repo in &config.repositories {
                info!("  {}", repo.name);
                info!("     Repository: {}/{}", repo.owner, repo.repo);
                info!("     Version: {:?}", repo.version);
                if repo.optional {
                    info!("     Optional: true");
                }
                if !repo.release_assets.is_empty() {
                    info!("     Assets: {}", repo.release_assets.join(", "));
                }
                if !repo.repo_files.is_empty() {
                    info!("     Files: {}", repo.repo_files.join(", "));
                }
            }
        }

        Commands::Devices { remote_auth_url } => {
            use penumbra_installer::adb::AdbManager;

            info!("Checking device connection...");
            match AdbManager::connect(remote_auth_url).await {
                Ok(_) => {
                    info!("Single device connected and ready for installation");
                }
                Err(InstallerError::NoDevice) => {
                    warn!("No Android device connected");
                    warn!("   Please connect a device and enable USB debugging");
                    std::process::exit(1);
                }
                Err(InstallerError::MultipleDevices) => {
                    warn!("Multiple devices connected");
                    warn!("   Please connect exactly one device for installation");
                    std::process::exit(1);
                }
                Err(e) => {
                    error!("ADB connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::DumpLogs {
            stream,
            remote_auth_url,
        } => dump_logcat_and_exit(stream, remote_auth_url).await,
    }

    Ok(())
}

fn parse_variable_overrides(tokens: &[String]) -> Result<HashMap<String, String>> {
    let mut overrides = HashMap::new();
    let mut pending_name: Option<String> = None;

    for token in tokens {
        if let Some(name) = pending_name.take() {
            // We previously saw a flag, this token is the value
            if token.starts_with("--") {
                return Err(InstallerError::CLI(format!(
                    "Variable flag '--{name}' missing value. Followed by '{token}'"
                )));
            }

            overrides.insert(name, token.clone());
            continue;
        }

        if token == "--" {
            // Ignore explicit separators
            continue;
        }

        if let Some(stripped) = token.strip_prefix("--") {
            if stripped.is_empty() {
                return Err(InstallerError::CLI("Variable flag cannot be empty".into()));
            }

            if let Some((name, value)) = stripped.split_once('=') {
                if name.trim().is_empty() {
                    return Err(InstallerError::CLI(
                        "Variable flag name cannot be empty".into(),
                    ));
                }
                overrides.insert(name.trim().to_string(), value.to_string());
            } else {
                pending_name = Some(stripped.to_string());
            }
        } else {
            return Err(InstallerError::CLI(format!(
                "Unexpected variable token '{}'. Variable flags must start with '--'",
                token
            )));
        }
    }

    if let Some(name) = pending_name {
        return Err(InstallerError::CLI(format!(
            "Variable flag '--{}' requires a value",
            name
        )));
    }

    Ok(overrides)
}
