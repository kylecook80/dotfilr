use crate::error;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {}

const CONFIG_NAME: &str = "dotfilr.toml";
static FILES: OnceLock<Vec<PathBuf>> = OnceLock::new();

impl Config {
    /// Reads config from /etc/mercury.toml and parses it to a struct.
    // In theory we can have an option arg here to accept a config passed via --flag at runtime
    pub fn new(file: Option<PathBuf>) -> Result<Config, error::Error> {
        let mut v = Vec::new();
        v.push(Path::new("./").join(CONFIG_NAME));
        v.push(Path::new("/etc").join(CONFIG_NAME));
        let _ = FILES.set(v);

        let mut config_path;
        if let Some(p) = file {
            config_path = p;
        } else {
            config_path = PathBuf::from(FILES.get().unwrap().last().unwrap());

            for file in FILES.get().unwrap() {
                if Path::new(file).exists() {
                    let pathbuf = PathBuf::from(file);
                    config_path = pathbuf;
                }
            }
        }

        if !config_path.exists() {
            return Err(error::Error::ConfigOpen(config_path));
        }

        let mut config_file = File::open(config_path)?;
        let mut buf = String::new();
        let _bytes = config_file.read_to_string(&mut buf);
        let res: Result<Config, toml::de::Error> = toml::from_str(&buf);

        match res {
            Ok(t) => Ok(t),
            Err(e) => Err(error::Error::ConfigParse(e)),
        }
    }
}
