//! Library for parsing and working with KIWAD archives.

#![deny(rust_2018_idioms, rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_op_in_unsafe_fn)]

use std::collections::hash_map::Entry;
use std::collections::HashMap;

mod archive;
pub use self::archive::Archive;

pub mod crc;

mod interner;
pub use self::interner::*;

pub mod types;

mod parse;

/// An archive that uses an [`Interner`] to automatically
/// intern files on access.
pub struct InternedArchive {
    archive: Archive,
    interner: Interner,
    handles: HashMap<&'static str, FileHandle>,
}

impl InternedArchive {
    /// Consumes an [`Archive`] and creates a new one which
    /// automatically interns accessed files.
    pub fn new(archive: Archive) -> Self {
        Self {
            archive,
            interner: Interner::new(),
            handles: HashMap::new(),
        }
    }

    /// Resets the state of interned files in the archive.
    pub fn reset(&mut self) {
        self.interner.invalidate_all();
        self.handles.clear();
    }

    /// Attempts to retrieve the **decompressed** contents of
    /// `file` from the archive.
    ///
    /// Note that decompression will only happen on first
    /// access. Decompressed data will be cached and are
    /// cheap to fetch subsequently.
    ///
    /// This may fail if interning it fails internally.
    pub fn get(&mut self, file: &'static str) -> anyhow::Result<&[u8]> {
        match self.handles.entry(file) {
            Entry::Occupied(entry) => {
                // The requested file is already interned, we
                // retrieve its handle and return the contents.
                let handle = *entry.get();

                // We manage the Interner and never expose direct
                // access to it. If we get a handle, it's valid.
                Ok(self.interner.fetch(handle).unwrap())
            }

            Entry::Vacant(entry) => {
                // The file is not yet interned, we do that first.
                let handle = self.interner.intern(&self.archive, file)?;
                entry.insert(handle);

                // We manage the Interner and never expose direct
                // access to it. If we get a handle, it's valid.
                Ok(self.interner.fetch(handle).unwrap())
            }
        }
    }
}
