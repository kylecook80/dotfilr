mod config;
mod directories;
mod error;

use crate::config::Config;
use crate::directories::HomeDir;

use std::fs::{DirBuilder, read_dir, remove_file};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use git2::Repository;
use reqwest::Url;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    name: Option<String>,

    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install(InstallArgs),
    Uninstall(UninstallArgs),
    Push(PushArgs),
}

#[derive(Args)]
struct InstallArgs {
    #[arg(long)]
    git: Option<String>,

    #[arg(long)]
    path: Option<PathBuf>,
}

#[derive(Args)]
struct UninstallArgs {

}

#[derive(Args)]
struct PushArgs {

}

fn make_config_dir(path: &str) -> Result<PathBuf> {
    let home = home::home_dir();
    let dotfilr;
    
    if let Some(h) = home {
        dotfilr = h.join(".dotfilr");
        DirBuilder::new()
            .recursive(true)
            .create(&dotfilr)?;
    } else {
        return Err(anyhow!("Error: Could not get home directory."));
    }

    let full_path = dotfilr.join(path);

    DirBuilder::new()
        .recursive(true)
        .create(full_path.clone())?;

    Ok(full_path)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Read Config File
    let mut config: error::Result<Config>;

    if let Some(c) = cli.config {
        config = Config::new(Some(c));
    } else {
        config = Config::new(None);
    }

    config = Config::new(None);
    if config.is_err() {
        let err = config.err().unwrap();
        return Err(anyhow!("Error: {:?}", err));
    }

    let config = config.unwrap();

    // Home directory path
    let home_dir = HomeDir::new();

    // Ensure main config directory is created
    let dotfilr_path = make_config_dir(".")?;

    // Match commands
    match cli.command {
        Commands::Install(args) => {
            let repos_path = make_config_dir("repos")?;
            
            if args.git.is_some() && args.path.is_some() {
                return Err(anyhow!("Error: Can only use one of --git or --path."));
            }

            if let Some(g) = args.git {
                let url = Url::parse(&g)?;
                let repo_path = make_config_dir(&format!("repos/{}", url.path()))?;
                let repo = Repository::clone(&url.to_string(), &repo_path)?;
            }

            if let Some(p) = args.path {
                // Create bin directory if it doesn't already exist
                let home_bin = make_home_dir("bin")?;

                // Install bin files
                let bin_path = p.join("bin");
                for item in read_dir(bin_path)? {
                    let entry = item?;
                    let dest = home_bin.join(entry.file_name());

                    if Path::exists(&dest) {
                        println!("Skipping {}", dest.as_os_str().to_str().unwrap());    
                    } else {
                        symlink(entry.path(), dest)?;
                    }
                }
            }
        },
        Commands::Uninstall(args) => {
            let home_bin = home::home_dir().unwrap().join("bin");
            for item in read_dir(home_bin)? {
                let entry = item?;
                if entry.metadata()?.is_symlink() {
                    remove_file(entry.path())?;
                }
            }
        },
        Commands::Push(args) => {

        }
    }

    Ok(())
}
