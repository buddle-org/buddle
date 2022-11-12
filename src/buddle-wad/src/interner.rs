//! Interner for efficiently handling decompression and
//! access to files inside archives.

use anyhow::{anyhow, bail};
use flate2::{Decompress, FlushDecompress, Status};

use crate::archive::Archive;

/// A file handle which refers to an interned archive file
/// in an [`Interner`].
///
/// Handles can be used with [`Interner::fetch`] to retrieve
/// the file contents associated with them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileHandle(u32);

/// Facilitates decompressing and accessing [`Archive`]
/// files by assigning them [`FileHandle`]s.
///
/// Interning a file loads its **uncompressed** contents
/// into an internal buffer and conveniently assigns it a
/// [`FileHandle`] for later retrieval.
///
/// By design, handles can only be invalidated all at once.
/// Specific invalidations per handle are not intended or
/// supported.
pub struct Interner {
    // A dynamically grown buffer which stores all decompressed
    // file data in it as a contagious stream of data.
    buffer: Vec<u8>,

    // Stores the start offsets into `buffer` for every individual
    // file stream.
    //
    // We use indices into this list as the file access handles.
    starts: Vec<usize>,

    // The zlib object for data decompression.
    decompress: Decompress,
}

impl Interner {
    /// Creates a new, empty interner.
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            starts: Vec::new(),
            decompress: Decompress::new(true),
        }
    }

    /// Invalidates all currently interned files and their
    /// associated [`FileHandle`]s.
    ///
    /// After this operation, no previously interned file
    /// can be retrieved anymore.
    ///
    /// The memory allocations will be preserved.
    pub fn invalidate_all(&mut self) {
        self.buffer.clear();
        self.starts.clear();
    }

    /// Interns a file named `file` from the `archive`.
    ///
    /// This will return the [`FileHandle`] associated
    /// with the file on success, or an error in one
    /// of the following conditions:
    ///
    /// - `file` does not exist inside the archive
    /// - for compressed files, decompressing it fails
    pub fn intern(&mut self, archive: &Archive, file: &str) -> anyhow::Result<FileHandle> {
        let raw_archive = archive.raw_archive();
        let file = archive
            .journal()
            .find(file)
            .ok_or_else(|| anyhow!("{} is not in the archive", file))?;

        // Make a new, unique handle out of the index for
        // the metadata we are going to store for the file.
        let handle = FileHandle(self.starts.len() as u32);

        // Remember where this file is starting in memory.
        let file_start = self.buffer.len();
        self.starts.push(file_start);

        let data = file.extract(raw_archive);
        if file.compressed {
            // Decompress the data to the end of our internal buffer.
            self.decompress_to_end(data, file_start)?;
        } else {
            // The file is not compressed, so we just
            // extend the data buffer with it.
            self.buffer.extend_from_slice(file.extract(raw_archive));
        }

        Ok(handle)
    }

    /// Fetches interned file data given the [`FileHandle`]
    /// obtained from [`Interner::intern`].
    ///
    /// The returned slice always contains decompressed file
    /// contents.
    ///
    /// Returns [`None`] if the mapping was previously
    /// invalidated and not populated again.
    pub fn fetch(&self, handle: FileHandle) -> Option<&[u8]> {
        let idx = handle.0 as usize;
        self.starts.get(idx).map(|&start| {
            let end = self
                .starts
                .get(idx + 1)
                .copied()
                .unwrap_or(self.buffer.len());

            &self.buffer[start..end]
        })
    }

    fn decompress_to_end(&mut self, data: &[u8], start: usize) -> anyhow::Result<()> {
        // Reserve enough memory for decompressing the file.
        self.buffer.resize(start + data.len(), 0);

        // Decompress the data into the internal buffer.
        if self
            .decompress
            .decompress(data, &mut self.buffer[start..], FlushDecompress::Finish)?
            != Status::StreamEnd
        {
            bail!("Received incomplete zlib stream or wrong size expectation");
        }

        // Reset decompress object for next usage.
        self.decompress.reset(true);

        // Return the data we decompressed.
        Ok(())
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}
