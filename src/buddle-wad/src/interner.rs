use anyhow::{anyhow, bail};
use flate2::{Decompress, FlushDecompress, Status};

use crate::archive::Archive;

/// A handle to a given file in an [`Interner`].
///
/// Handles can be used with [`Interner::fetch`] to retrieve the file
/// contents associated with them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileHandle(u32, u32);

/// An interner that provides convenient access to files from
/// an [`Archive`].
///
/// Interning a file loads its **uncompressed** contents into an
/// internal buffer and conveniently assigns it a [`FileHandle`]
/// for later retrieval.
///
/// [`Interner`]s are tied to the [`Archive`] they are constructed
/// from, and may not hold files from other archives.
///
/// By design, handles can only be invalidated all at once. For
/// optimizing memory usage, only intern files when you need to use
/// them and flush after you don't need the files anymore.
pub struct Interner<A> {
    archive: A,
    inner: InnerInterner,
}

impl<A: AsRef<Archive>> Interner<A> {
    /// Creates a new, empty interner.
    pub fn new(archive: A) -> Self {
        Self {
            archive,
            inner: InnerInterner {
                invalidation_count: 0,
                buf: Vec::new(),
                ends: Vec::new(),
                inflater: Decompress::new(true),
            },
        }
    }

    /// Invalidates all currently interned files and their associated
    /// [`FileHandle`]s.
    ///
    /// After this operation, no lookup for previously issued handles
    /// will succeed anymore.
    ///
    /// Memory allocations from previous usage of the [`Interner`] will
    /// be preserved for re-use.
    pub fn invalidate_all(&mut self) {
        self.inner.invalidation_count += 1;
        self.inner.buf.clear();
        self.inner.ends.clear();
    }

    /// Fetches interned file data given the corresponding [`FileHandle`].
    ///
    /// The returned slice always contains decompressed file contents.
    ///
    /// Returns [`None`] if the mapping was invalidated.
    pub fn fetch(&self, handle: FileHandle) -> Option<&[u8]> {
        let inner = &self.inner;
        (handle.1 == inner.invalidation_count).then(|| {
            let idx = handle.0 as usize;

            let start = inner.ends.get(idx.wrapping_sub(1)).copied().unwrap_or(0);
            let end = inner.ends[idx];

            &inner.buf[start..end]
        })
    }

    /// Interns a file named `file` from the [`Archive`].
    ///
    /// Returns the [`FileHandle`] associated with the file on success,
    /// for later retrieval.
    ///
    /// This method may fail for several reasons:
    ///
    /// - `file` does not exist in the archive
    ///
    /// - decompressing the file falied, either due ot invalid data or
    ///   invalid encoded size expectations
    pub fn intern(&mut self, file: &str) -> anyhow::Result<FileHandle> {
        self.inner.intern(self.archive.as_ref(), file)
    }
}

struct InnerInterner {
    // The number of times the interner state was invalidated.
    // This is tracked so previously issued file handles cannot
    // accidentally fetch garbage after invalidation anymore.
    invalidation_count: u32,

    // A dynamically grown buffer which stores all decompressed file
    // data as a contagious stream of bytes.
    buf: Vec<u8>,

    // Stores the end offsets into `buffer` for every individual file.
    //
    // We use indices into this list as the file access handles.
    ends: Vec<usize>,

    // The zlib inflater state for data decompression.
    inflater: Decompress,
}

impl InnerInterner {
    fn intern(&mut self, archive: &Archive, file: &str) -> anyhow::Result<FileHandle> {
        let raw_archive = archive.raw_archive();
        let file = archive
            .journal()
            .find(file)
            .ok_or_else(|| anyhow!("'{file}' is not in the archive"))?;

        // Make a new handle for the file. Archives have an upper limit of
        // u32 files, so by design this can never produce ambiguous values.
        let handle = FileHandle(self.ends.len() as u32, self.invalidation_count);

        // Remember where this file is ending in memory.
        let size_hint = file.uncompressed_size as usize;
        self.ends.push(self.buf.len() + size_hint);

        // Extract the file contents from the archive data.
        let data = file.extract(raw_archive);
        if file.compressed {
            // Decompress the data into our internal buffer.
            self.decompress_to_buf(data, size_hint)?;
        } else {
            // The file is not compressed, so we just grow the buffer.
            self.buf.extend_from_slice(data);
        }

        Ok(handle)
    }

    fn decompress_to_buf(&mut self, data: &[u8], hint: usize) -> anyhow::Result<()> {
        // Reserve enough memory for decompressing the file.
        let start = self.buf.len();
        self.buf.resize(start + hint, 0);

        // Decompress the data into the internal buffer.
        if self
            .inflater
            .decompress(data, &mut self.buf[start..], FlushDecompress::Finish)?
            != Status::StreamEnd
        {
            bail!("received incomplete zlib stream or wrong size expectation");
        }

        // Reset decompress object for next usage.
        self.inflater.reset(true);

        Ok(())
    }
}
