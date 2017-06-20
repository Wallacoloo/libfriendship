use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use byteorder::{LittleEndian, ReadBytesExt};
use digest::Digest;
use filebuffer::FileBuffer;
use sha2::Sha256;

use routing::EffectId;


/// Resource manager. Where to search for various file types (e.g. Effects).
/// Uses a 'dumb' implementation - doesn't try to auto-configure paths (/usr/bin/share/[...],
/// ~/.friendship, etc). Instead, designed to be configured by the host.
#[derive(Default, Debug)]
pub struct ResMan {
    /// List of directories to search for files in.
    dirs: Vec<PathBuf>,
    /// Object that handles indexing/caching files.
    cache: RefCell<ResCache>,
}

/// Audio that may be on-disk.
#[derive(Clone, Debug)]
pub struct AudioBuffer {
    buffer: Rc<FileBuffer>,
}

#[derive(Default, Debug)]
struct ResCache {
    /// Map sha's to paths.
    sha256_to_path: HashMap<[u8; 32], PathBuf>,
}

impl ResMan {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn add_dir(&mut self, dir: PathBuf) {
        self.dirs.push(dir);
    }
    /// Returns all definitions of the given effect in the form of an iterator
    ///   over boxed objects implementing io::Read.
    pub fn find_effect<'a>(&'a self, id: &'a EffectId) -> impl Iterator<Item=(PathBuf, File)> + 'a {
        self.iter_effect_files(id).map(|path| {
            (path.clone(), File::open(path).unwrap())
        })
    }
    fn iter_effect_files<'a>(&'a self, id: &'a EffectId) -> impl Iterator<Item=PathBuf> + 'a {
        self.iter_all_files(id.sha256().as_ref()).filter(move |f| {
            let did_match = match *id.sha256() {
                None => true,
                Some(ref hash) => {
                    let mut file = File::open(f).unwrap();
                    // TODO: the hash could still change between now and when we parse the file!
                    let result = Sha256::digest_reader(&mut file).unwrap();
                    // Cache this sha256->file relationship.
                    self.cache.borrow_mut().notify_sha256(f.clone(), slice_to_array32(result.as_slice()));
                    hash == result.as_slice()
                }
            };
            trace!("Resman: testing hash for: {:?} ({:?})", f, did_match);
            did_match
        })
    }
    /// Iterates over all files.
    /// Files with matching search criteria are iterated first.
    /// Files may be visited multiple times. This happens if their sha matches the hint.
    fn iter_all_files<'a>(&'a self, sha256_hint: Option<&[u8; 32]>) -> impl Iterator<Item=PathBuf> + 'a {
        let prioritized = sha256_hint
            .and_then(|sha| self.cache.borrow().get_path_by_sha256(sha).cloned())
            .into_iter();
        // dirs as PathBuf -> valid ReadDir objects
        let all_files = self.dirs.iter().filter_map(|dir_path| {
            fs::read_dir(dir_path)
                .map_err(|e| warn!("ResMan: Failed to read directory {:?}: {}", dir_path, e))
                .ok()
        })
        // ReadDir objects -> flat list of Result<DirEntry>
        .flat_map(|read_dir| {
            read_dir
        })
        // Result<DirEntry> -> DirEntry
        .filter_map(|dir_entry| {
            dir_entry
                .map_err(|e| warn!("ResMan: Failed to read directory entry: {}", e))
                .ok()
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
        });
        prioritized.chain(all_files)
    }
}


impl AudioBuffer {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        if path.as_ref().extension().map(|e| e == "f32").unwrap_or(false) {
            // TODO: don't abort on failure; instead, treat as zero stream
            Ok(Self {
                buffer: Rc::new(FileBuffer::open(path)?)
            })
        } else {
            Err(io::Error::new(io::ErrorKind::Other, format!("Unknown audio format for file: {:?}", path.as_ref())))
        }
    }
    /// Read data from the buffer.
    pub fn get(&self, idx: u64, ch: u32) -> f32 {
        assert_eq!(ch, 0);
        // TODO: this isn't very dependable for 32-bit OSes.
        let idx = idx*4; // frame index -> byte index
        let view = &self.buffer[idx as usize..idx as usize + 4];
        let mut reader = Cursor::new(view);
        // Read float or 0f32 if error (e.g. end of file?)
        reader.read_f32::<LittleEndian>().unwrap_or(0f32)
    }
}

impl ResCache {
    /// Call upon discovery of a file's hash.
    fn notify_sha256(&mut self, path: PathBuf, sha256: [u8; 32]) {
        self.sha256_to_path.insert(sha256, path);
    }
    /// Attempt to look up a file by its hash.
    fn get_path_by_sha256(&self, sha256: &[u8; 32]) -> Option<&PathBuf> {
        self.sha256_to_path.get(sha256)
    }
}

/// Create a 32-entry array from a slice.
fn slice_to_array32(slice: &[u8]) -> [u8; 32] {
    let mut ret: [u8; 32] = Default::default();
    ret.copy_from_slice(slice);
    ret
}
