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
    },
    Custom {
        config: PathBuf,
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
    },
    Url {
        url: String,
        #[arg(long, value_delimiter = ',')]
        repos: Option<Vec<String>>,
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
        Commands::Install { repos } => {
            let config = ConfigLoader::load_builtin("penumbra")?;
            run_installation(config, repos.as_deref()).await?;
        }

        Commands::Custom { config, repos } => {
            let config = ConfigLoader::load_from_file(&config).await?;
            run_installation(config, repos.as_deref()).await?;
        }

        Commands::Url { url, repos } => {
            let config = ConfigLoader::load_from_url(&url).await?;
            run_installation(config, repos.as_deref()).await?;
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

async fn run_installation(
    config: penumbra_installer::InstallConfig,
    repos: Option<&[String]>,
) -> Result<()> {
    if let Some(repo_names) = repos {
        let filtered = config.filter_repositories(repo_names)?;
        println!(
            "Installing {} of {} repositories:",
            filtered.len(),
            config.repositories.len()
        );
        for repo in &filtered {
            println!("  - {}", repo.name);
        }
        println!();
    }

    let mut engine = InstallationEngine::new(config).await?;
    engine.install(repos).await?;

    Ok(())
}
