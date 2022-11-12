//! In-memory representations of WAD archives.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io, mem,
    path::Path,
};

use memmap2::{Mmap, MmapOptions};

use crate::{
    interner::{FileHandle, Interner},
    types as wad_types,
};

/// A read-only archive that is either memory-mapped or
/// allocated in heap memory.
///
/// In the interest of performance optimization, control
/// over how a file is processed is handed to the user.
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
    /// `verify_crc` will optionally validate all encoded
    /// CRCs in the archive file when `true`.
    ///
    /// This is the preferred option of working with relatively
    /// small files but it's always best to profile.
    pub fn heap<P: AsRef<Path>>(path: P, verify_crc: bool) -> anyhow::Result<Self> {
        HeapArchive::open(path, verify_crc).map(Self::Heap)
    }

    /// Opens a file at the given `path` and maps it into
    /// memory without copying the data.
    ///
    /// The file will be kept open for the entire lifetime
    /// of the [`Archive`] object to keep the mapping intact.
    ///
    /// `verify_crc` will optionally validate all encoded
    /// CRCs in the archive file when `true`.
    ///
    /// This is the preferred option of working with relatively
    /// large files but it's always best to profile.
    pub fn mmap<P: AsRef<Path>>(path: P, verify_crc: bool) -> anyhow::Result<Self> {
        MemoryMappedArchive::open(path, verify_crc).map(Self::MemoryMapped)
    }

    #[inline]
    pub(crate) fn journal(&self) -> &Journal {
        match self {
            Self::MemoryMapped(a) => &a.journal,
            Self::Heap(a) => &a.journal,
        }
    }

    #[inline]
    pub(crate) fn raw_archive(&self) -> &[u8] {
        match self {
            Self::MemoryMapped(a) => &a.mapping,
            Self::Heap(a) => &a.data,
        }
    }

    /// Gets the raw contents of an archived file by its
    /// encoded name string.
    ///
    /// Returns [`None`] when no such file exists in the
    /// archive.
    pub fn file_raw(&self, name: &str) -> Option<&[u8]> {
        self.journal()
            .find(name)
            .map(|f| f.extract(self.raw_archive()))
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

    fn build_from(&mut self, archive: wad_types::Archive) {
        archive.files.into_iter().for_each(|f| self.insert(f));
    }

    pub fn find(&self, file: &str) -> Option<&wad_types::File> {
        self.inner.get(file)
    }
}

pub struct MemoryMappedArchive {
    // We internally hold the file so it stays open for the
    // lifetime of the memory mapping.
    //
    // Unmapped before the underlying file is closed.
    mapping: Mmap,

    // The owned file that backs the archive.
    //
    // Closed after the mapping is dropped.
    #[allow(unused)]
    file: File,

    journal: Journal,
}

impl MemoryMappedArchive {
    fn open<P: AsRef<Path>>(path: P, verify_crc: bool) -> anyhow::Result<Self> {
        // Open the file for the given path.
        let file = File::open(path)?;

        // Map it into memory.
        let mut this = Self {
            // SAFETY: We own the file and WAD archives are generally
            // treated as read-only by the game, we most likely won't
            // run into any realistic synchronization conflicts with it.
            mapping: unsafe { MmapOptions::new().populate().map(&file)? },
            file,
            journal: Journal {
                inner: BTreeMap::new(),
            },
        };

        // Parse the archive and build the file journal.
        let archive = wad_types::Archive::parse(&mut io::Cursor::new(&this.mapping))?;
        if verify_crc {
            archive.verify_crcs(&this.mapping)?;
        }
        this.journal.build_from(archive);

        Ok(this)
    }
}

pub struct HeapArchive {
    journal: Journal,
    data: Vec<u8>,
}

impl HeapArchive {
    fn open<P: AsRef<Path>>(path: P, verify_crc: bool) -> anyhow::Result<Self> {
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
        if verify_crc {
            archive.verify_crcs(&this.data)?;
        }
        this.journal.build_from(archive);

        Ok(this)
    }
}
