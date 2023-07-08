use crate::error::{Error, Result};

use std::fs::{DirBuilder, DirEntry, read_dir, ReadDir};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

pub struct ManagedDirectory {
    path: PathBuf,
}

pub struct DirIter(ReadDir);

impl ManagedDirectory {
    pub fn new(path: PathBuf) -> Self {
        ManagedDirectory {
            path
        }
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    pub fn to_string(&self) -> String {
        self.path.to_str().unwrap().to_string()
    }

    pub fn exists(&self) -> bool {
        Path::exists(&self.path)
    }

    pub fn subdir(&mut self, path: String) -> ManagedDirectory {
        let subdir_path = self.path.join(path);
        self.create_dir(&subdir_path);
        
        ManagedDirectory {
            path: subdir_path
        }
    }

    pub fn create_dir(&mut self, path: &Path) -> Result<PathBuf> {
        let new_path = self.path.join(path);
        DirBuilder::new()
            .recursive(true)
            .create(&new_path)?;
        Ok(new_path)
    }

    pub fn link(&mut self, file: &Path) -> Result<PathBuf> {
        let new_link = self.path.join(file.file_name().unwrap().to_str().unwrap());
        if Path::exists(&new_link) {
            Err(Error::FileExists)
        } else {
            symlink(file, &new_link)?;
            Ok(new_link)
        }
    }

    pub fn install_to(&mut self, dir: &Path) -> Result<PathBuf> {
        let path_dir = self.subdir(String::from("bin"));
        let mut home_dir = home_dir.subdir(String::from("bin"));

        for item in path_dir.iter() {
            let entry = item?;

            let dest = home_dir.link(&entry.path());
            if let Err(Error::FileExists) = dest {
                println!("File exists. Skipping {}", entry.file_name().to_str().unwrap());
            } else {
                println!("Linking new file {}", entry.file_name().to_str().unwrap());
            }
        }

        Ok(path_dir)
    }

    pub fn iter(&self) -> DirIter {
        let dir = read_dir(&self.path).unwrap();
        DirIter(dir)
    }
}

impl Iterator for DirIter {
    type Item = std::result::Result<DirEntry, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next() {
            Some(t) => {
                return Some(t)
            },
            None => {
                return None;
            }
        }
    }
}
