use std::path::PathBuf;
use std::io;
use std::io::Cursor;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;

use byteorder::{LittleEndian, ReadBytesExt};
use digest::digest_reader;
use filebuffer::FileBuffer;
use sha2::Sha256;

use routing::EffectId;


/// Resource manager. Where to search for various file types (e.g. Effects).
/// Uses a 'dumb' implementation - doesn't try to auto-configure paths (/usr/bin/share/[...],
/// ~/.friendship, etc). Instead, designed to be configured by the host.
#[derive(Default, Debug)]
pub struct ResMan {
    dirs: Vec<PathBuf>,
}

/// Audio that may be on-disk.
// TODO: derive Debug. FileBuffer doesn't have it implemented.
#[derive(Clone)]
pub struct AudioBuffer {
    buffer: Rc<FileBuffer>,
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
        self.iter_all_files().filter(move |f| {
            trace!("Resman: testing hash for: {:?}", f);
            match *id.sha256() {
                None => true,
                Some(ref hash) => {
                    let mut file = File::open(f).unwrap();
                    let result = digest_reader::<Sha256>(&mut file).unwrap();
                    hash == result.as_slice()
                }
            }
        })
    }
    fn iter_all_files<'a>(&'a self) -> impl Iterator<Item=PathBuf> + 'a {
        // dirs as PathBuf -> valid ReadDir objects
        self.dirs.iter().filter_map(|dir_path| {
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
        })
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
