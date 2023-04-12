use std::{
    collections::BTreeMap,
    fs::{self, File},
    io, mem,
    path::Path,
};

use memmap2::{Mmap, MmapOptions};

use crate::types as wad_types;

pub struct Archive(ArchiveInner);

enum ArchiveInner {
    MemoryMapped(MemoryMappedArchive),
    Heap(HeapArchive),
}

impl Archive {
    /// Opens a file at the given `path` and operates on it
    /// from heap-allocated memory.
    ///
    /// The file handle will be closed immediately after reading.
    ///
    /// `verify_crc` will optionally run validation of all encoded
    /// CRCs in archive files when `true`.
    ///
    /// This is the preferred option of working with relatively small
    /// files but it's always best to profile.
    pub fn heap<P: AsRef<Path>>(path: P, verify_crc: bool) -> anyhow::Result<Self> {
        HeapArchive::open(path, verify_crc).map(|a| Self(ArchiveInner::Heap(a)))
    }

    /// Opens a file at the given `path` and operates on it
    /// from a memory mapping.
    ///
    /// The file handle will be kept open for the entire lifetime
    /// of the [`Archive`] object.
    ///
    /// `verify_crc` will optionally run validation of all encoded
    /// CRCs in the archive files when `true`.
    ///
    /// The is the preferred option of working with relatively large
    /// files but it's always best to profile.
    pub fn mmap<P: AsRef<Path>>(path: P, verify_crc: bool) -> anyhow::Result<Self> {
        MemoryMappedArchive::open(path, verify_crc).map(|a| Self(ArchiveInner::MemoryMapped(a)))
    }

    #[inline]
    pub(crate) fn journal(&self) -> &Journal {
        match &self.0 {
            ArchiveInner::MemoryMapped(a) => &a.journal,
            ArchiveInner::Heap(a) => &a.journal,
        }
    }

    #[inline]
    pub(crate) fn raw_archive(&self) -> &[u8] {
        match &self.0 {
            ArchiveInner::MemoryMapped(a) => &a.mapping,
            ArchiveInner::Heap(a) => &a.data,
        }
    }

    /// Gets the number of files in the archive.
    #[inline]
    pub fn len(&self) -> usize {
        self.journal().inner.len()
    }

    /// Whether the archive is empty, i.e. does not contain any files.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the raw contents of an archived file by its string name.
    ///
    /// This returns a `(is_compressed, contents)` tuple to give the
    /// user maximum flexibility in how to process the results.
    ///
    /// Returns [`None`] when no file named `name` exists in the archive.
    pub fn file_raw(&self, name: &str) -> Option<(bool, &[u8])> {
        self.journal()
            .find(name)
            .map(|f| (f.compressed, f.extract(self.raw_archive())))
    }
}

impl AsRef<Archive> for Archive {
    #[inline]
    fn as_ref(&self) -> &Archive {
        self
    }
}

pub(crate) struct Journal {
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

struct MemoryMappedArchive {
    // Internally kept memory mapping of the archive file contents.
    //
    // By guaranteed drop order, this will be unmapped before the
    // file below is closed.
    mapping: Mmap,

    // The backing file of the above mapping.
    //
    // Owned by this structure so the mapping never becomes invalid.
    // Closed when this structure is dropped.
    #[allow(unused)]
    file: File,

    // The journal of files in the archive.
    journal: Journal,
}

impl MemoryMappedArchive {
    fn open<P: AsRef<Path>>(path: P, verify_crc: bool) -> anyhow::Result<Self> {
        // Attempt to open the file at the given path.
        let file = File::open(path)?;
        let mut this = Self {
            // SAFETY: We own the file and keep it around until the mapping
            // is closed; see comments in `MemoryMappedArchive` above.
            //
            // Since archive files are generally treated as read-only by us
            // and most other applications, we likely won't run into any
            // synchronization conflicts we need to account for.
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

struct HeapArchive {
    // The journal of files in the archive.
    journal: Journal,

    // The raw archive data, allocated on the heap.
    data: Box<[u8]>,
}

impl HeapArchive {
    fn open<P: AsRef<Path>>(path: P, verify_crc: bool) -> anyhow::Result<Self> {
        // Attempt to read the given file into a byte vector.
        let mut this = Self {
            journal: Journal {
                inner: BTreeMap::new(),
            },
            data: fs::read(path)?.into_boxed_slice(),
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
