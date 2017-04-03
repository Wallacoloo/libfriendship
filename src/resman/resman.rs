extern crate sha2;

use std::path::PathBuf;
use std::io::Read;
use std::fs;
use std::fs::File;

use self::sha2::Sha256;
use self::sha2::Digest;

use super::super::routing::EffectDesc;


/// Resource manager. Where to search for various file types (e.g. Effects).
/// Uses a 'dumb' implementation - doesn't try to auto-configure paths (/usr/bin/share/[...],
/// ~/.friendship, etc). Instead, designed to be configured by the host.
struct ResMan {
    dirs: Vec<PathBuf>,
}

impl ResMan {
    pub fn new() -> Self {
        Self {
            dirs: Vec::new(),
        }
    }
    pub fn add_dir(&mut self, dir: PathBuf) {
        self.dirs.push(dir);
    }
    pub fn find_effect<'a>(&'a self, desc: &'a EffectDesc) -> impl Iterator<Item=PathBuf> + 'a{
        self.iter_all_files().filter(move |f| {
            match desc.sha256() {
                &None => true,
                &Some(hash) => hash == self.file_sha256_hash(f),
            }
        })
    }
    fn iter_all_files<'a>(&'a self) -> impl Iterator<Item=PathBuf> + 'a{
        // dirs as PathBuf -> valid ReadDir objects
        self.dirs.iter().filter_map(|dir_path| {
            fs::read_dir(dir_path).ok()
        })
        // ReadDir objects -> flat list of Result<DirEntry>
        .flat_map(|read_dir| {
            read_dir
        })
        // Result<DirEntry> -> DirEntry
        .filter_map(|dir_entry| {
            dir_entry.ok()
        })
        // keep only the files
        .filter(|dir_entry| {
            if let Ok(file_type)=dir_entry.file_type() {
                file_type.is_file()
            } else {
                false
            }
        })
        // DirEntry -> Path
        .map(|dir_entry| {
            dir_entry.path()
        })
    }

    /// Returns the sha256 hash of the file's contents.
    fn file_sha256_hash(&self, path: &PathBuf) -> [u8; 32] {
        // TODO: Rewrite surroundings to avoid the possibility that the file doesn't exist / has
        // been deleted since the time the directory was enumerated.
        let mut hasher = Sha256::new();
        let mut file_contents = Vec::new();
        hasher.input({
            File::open(path).unwrap().read_to_end(&mut file_contents);
            &file_contents
        });
        let mut res: [u8; 32] = Default::default();
        res.copy_from_slice(hasher.result().as_slice());
        res
    }
}
