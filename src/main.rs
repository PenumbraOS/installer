use clap::{Parser, Subcommand};
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
                InstallationEngine::new_with_cache(config, cache_path.clone()).await?
            } else {
                InstallationEngine::new(config).await?
            };

            if cache_dir.is_some() {
                engine.install_cached(repos).await?;
            } else {
                engine.install(repos).await?;
            }
        }

        Commands::Uninstall { repos } => {
            let config = ConfigLoader::load_builtin("penumbra")?;
            let mut engine = InstallationEngine::new(config).await?;
            engine.uninstall(repos).await?;
        }

        Commands::Download { repos, cache_dir } => {
            let config = ConfigLoader::load_builtin("penumbra")?;
            let mut engine = InstallationEngine::new_with_cache(config, cache_dir).await?;
            engine.download(repos).await?;
        }

        Commands::List { config } => {
            let config = if let Some(config_path) = config {
                ConfigLoader::load_from_file(&config_path).await?
            } else {
                ConfigLoader::load_builtin("penumbra")?
            };

            println!("Available repositories in '{}':", config.name);
            println!();
            for repo in &config.repositories {
                println!("  {}", repo.name);
                println!("     Repository: {}/{}", repo.owner, repo.repo);
                println!("     Version: {:?}", repo.version);
                if !repo.release_assets.is_empty() {
                    println!("     Assets: {}", repo.release_assets.join(", "));
                }
                if !repo.repo_files.is_empty() {
                    println!("     Files: {}", repo.repo_files.join(", "));
                }
                println!();
            }
        }

        Commands::Devices => {
            use penumbra_installer::adb::AdbManager;

            println!("Checking device connection...");
            match AdbManager::connect().await {
                Ok(_) => {
                    println!("Single device connected and ready for installation");
                }
                Err(InstallerError::NoDevice) => {
                    println!("No Android device connected");
                    println!("   Please connect a device and enable USB debugging");
                    std::process::exit(1);
                }
                Err(InstallerError::MultipleDevices) => {
                    println!("Multiple devices connected");
                    println!("   Please connect exactly one device for installation");
                    std::process::exit(1);
                }
                Err(e) => {
                    println!("ADB connection failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
