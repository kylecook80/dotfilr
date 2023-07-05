use crate::error::Result;

use std::fs::DirBuilder;
use std::path::{Path, PathBuf};

trait ManagedDirectory {
    fn get_path(&self) -> &Path;
    fn create(&mut self, path: &Path) -> Result<PathBuf>;
}

pub struct HomeDir {
    path: PathBuf,
}

impl HomeDir {
    pub fn new() -> Self {
        HomeDir {
            path: home::home_dir().unwrap()
        }
    }
}

impl ManagedDirectory for HomeDir {    
    fn get_path(&self) -> &Path {
        &self.path
    }

    fn create(&mut self, path: &Path) -> Result<PathBuf> {
        let new_path = self.path.join(path);
        DirBuilder::new()
            .recursive(true)
            .create(&new_path)?;
        Ok(new_path)
    }
}

pub struct ConfigDir {
    
}
