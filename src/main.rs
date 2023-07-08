mod config;
mod directories;
mod error;

use crate::config::Config;
use crate::directories::ManagedDirectory;
use crate::error::Error;

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
    let mut home_dir = ManagedDirectory::new(home::home_dir().unwrap());

    // Ensure main config directory is created
    let mut dotfilr_dir = ManagedDirectory::new(home_dir.get_path().to_path_buf());

    // Match commands
    match cli.command {
        Commands::Install(args) => {
            dotfilr_dir.subdir(String::from("repos"));
            
            if args.git.is_some() && args.path.is_some() {
                return Err(anyhow!("Error: Can only use one of --git or --path."));
            }

            if let Some(g) = args.git {
                let url = Url::parse(&g)?;
                let repo_path = make_config_dir(&format!("repos/{}", url.path()))?;
                let repo = Repository::clone(&url.to_string(), &repo_path)?;
            }

            if let Some(p) = args.path {
                if !Path::exists(&p) {
                    return Err(anyhow!("Error. Path does not exist: {}", p.to_str().unwrap()));
                }

                let mut dotfiles_dir = ManagedDirectory::new(p);

                dotfiles_dir.install_to(home_dir.get_path());

                let path_files_dir = dotfiles_dir.subdir(String::from("files"));
                let mut home_files_dir = home_dir.subdir(String::from("files"));

                for item in path_files_dir.iter() {
                    let entry = item?;

                    let dest = home_files_dir.link(&entry.path());
                    if let Err(Error::FileExists) = dest {
                        println!("File exists. Skipping {}", entry.file_name().to_str().unwrap());
                    } else {
                        println!("Linking new file {}", entry.file_name().to_str().unwrap());
                    }
                }
            }
        },
        Commands::Uninstall(args) => {
            let home_bin = home::home_dir().unwrap().join("bin");
            for item in read_dir(home_bin)? {
                let entry = item?;
                if entry.metadata()?.is_symlink() {
                    println!("Removing file {}", entry.file_name().to_str().unwrap());
                    remove_file(entry.path())?;
                }
            }
        },
        Commands::Push(args) => {

        }
    }

    Ok(())
}
