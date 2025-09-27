use clap::{Parser, Subcommand};
use log::{info, warn, error};
use std::path::PathBuf;
use tokio;

use penumbra_installer::{ConfigLoader, InstallationEngine, InstallerError, Result};

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
    },
    Uninstall {
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
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
    Devices,
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
        } => {
            let config = match (config, config_url) {
                (None, None) => ConfigLoader::load_builtin("penumbra"),
                (None, Some(config_url)) => ConfigLoader::load_from_url(&config_url).await,
                (Some(config_path), None) => ConfigLoader::load_from_file(&config_path).await,
                (Some(_), Some(_)) => {
                    return Err(InstallerError::CLI(
                        "`config` and `config_url` options are mutually exclusive".into(),
                    ));
                }
            }?;
            let mut engine = if let Some(ref cache_path) = cache_dir {
                InstallationEngine::new_with_cache(
                    config,
                    cache_path.clone(),
                    cli.github_token.clone(),
                    None,
                )
                .await?
            } else {
                InstallationEngine::new_with_token(config, cli.github_token.clone(), None).await?
            };

            if cache_dir.is_some() {
                engine.install(repos, true).await?;
            } else {
                engine.install(repos, false).await?;
            }
        }

        Commands::Uninstall { repos } => {
            let config = ConfigLoader::load_builtin("penumbra")?;
            let mut engine =
                InstallationEngine::new_with_token(config, cli.github_token.clone(), None).await?;
            engine.uninstall(repos).await?;
        }

        Commands::Download { repos, cache_dir } => {
            let config = ConfigLoader::load_builtin("penumbra")?;
            let mut engine = InstallationEngine::new_with_cache(
                config,
                cache_dir,
                cli.github_token.clone(),
                None,
            )
            .await?;
            engine.download(repos).await?;
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
                if !repo.release_assets.is_empty() {
                    info!("     Assets: {}", repo.release_assets.join(", "));
                }
                if !repo.repo_files.is_empty() {
                    info!("     Files: {}", repo.repo_files.join(", "));
                }
            }
        }

        Commands::Devices => {
            use penumbra_installer::adb::AdbManager;

            info!("Checking device connection...");
            match AdbManager::connect().await {
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
    }

    Ok(())
}
