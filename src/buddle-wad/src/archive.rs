//! In-memory representations of WAD archives.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io, mem,
    path::Path,
};

use memmap2::{Mmap, MmapOptions};

use crate::types as wad_types;

/// A read-only archive that is either memory-mapped or
/// allocated in heap memory.
///
/// In the interest of performance optimization, control over
/// how a file is processed is handed to the user.
///
/// For smaller files, the heap backend should always be
/// preferred over memory mappings.
pub enum Archive {
    MemoryMapped(MemoryMappedArchive),
    Heap(HeapArchive),
}

impl Archive {
    /// Opens a file at the given `path` and tries to parse
    /// it in heap memory.
    ///
    /// The file will be closed immediately after it was
    /// read.
    ///
    /// This is the preferred option of working with relatively
    /// small files but it's always best to profile.
    pub fn heap<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        HeapArchive::open(path).map(Self::Heap)
    }

    /// Opens a file at the given `path` and maps it into
    /// memory without copying the data.
    ///
    /// The file will be kept open for the entire lifetime
    /// of the [`Archive`] object to keep the mapping intact.
    ///
    /// This is the preferred option of working with relatively
    /// large files but it's always best to profile.
    pub fn mmap<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        MemoryMappedArchive::open(path).map(Self::MemoryMapped)
    }
}

pub struct Journal {
    inner: BTreeMap<String, wad_types::File>,
}

impl Journal {
    fn insert(&mut self, mut file: wad_types::File) {
        let name = mem::take(&mut file.name);
        self.inner.insert(name, file);
    }

    fn find(&self, file: &str) -> Option<&wad_types::File> {
        self.inner.get(file)
    }
}

pub struct MemoryMappedArchive {
    file: File,
    journal: Journal,
    mapping: Mmap,
}

impl MemoryMappedArchive {
    fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // Open the file for the given path.
        let file = File::open(path)?;

        // Map it into memory.
        let mut this = Self {
            // SAFETY: We own the file, it stays open for the whole
            // lifetime of the archive.
            mapping: unsafe { MmapOptions::new().populate().map(&file)? },
            file,
            journal: Journal {
                inner: BTreeMap::new(),
            },
        };

        // Parse the archive and build the file journal.
        let archive = wad_types::Archive::parse(&mut io::Cursor::new(&this.mapping))?;
        archive
            .files
            .into_iter()
            .for_each(|f| this.journal.insert(f));

        Ok(this)
    }
}

pub struct HeapArchive {
    journal: Journal,
    data: Vec<u8>,
}

impl HeapArchive {
    fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // Read the file at path into a vector.
        let data = fs::read(path)?;

        // Create the archive object.
        let mut this = Self {
            journal: Journal {
                inner: BTreeMap::new(),
            },
            data,
        };

        // Parse the archive and build the file journal.
        let archive = wad_types::Archive::parse(&mut io::Cursor::new(&this.data))?;
        archive
            .files
            .into_iter()
            .for_each(|f| this.journal.insert(f));

        Ok(this)
    }
}
