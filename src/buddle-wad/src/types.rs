//! Common types in the KIWAD format.

use anyhow::bail;
use binrw::{
    binread,
    io::{Read, Seek},
    BinReaderExt,
};

use crate::{crc, parse::parse_file_name};

/// The header of a WAD archive.
#[binread]
pub struct Header {
    /// The format version in use.
    pub version: u32,
    /// The total number of files stored in the archive.
    pub file_count: u32,
    /// The configuration flags associated with the
    /// archive.
    ///
    /// These are only present when [`Header::version`]
    /// is `2` or greater.
    #[br(if(version >= 2))]
    pub flags: Option<u8>,
}

/// Metadata for a file encoded in a WAD archive.
#[binread]
pub struct File {
    /// The starting offset of the file in the archive.
    pub offset: u32,
    /// The uncompressed size of the file contents.
    pub uncompressed_size: u32,
    /// The compressed size of the file contents.
    ///
    /// When the file is stored uncompressed, this
    /// can be ignored.
    pub compressed_size: u32,
    /// Whether the file is stored compressed.
    #[br(map = |x: u8| x != 0)]
    pub compressed: bool,
    /// The CRC32 checksum of uncompressed file contents.
    pub crc: u32,
    #[br(temp)]
    name_len: u32,
    /// The name of the file in the archive.
    #[br(args(name_len as usize), parse_with = parse_file_name)]
    pub name: String,
}

impl File {
    /// Gets the size of the data described by this file.
    pub fn size(&self) -> usize {
        if self.compressed {
            self.compressed_size as usize
        } else {
            self.uncompressed_size as usize
        }
    }

    /// Extracts this file from the given raw archive bytes.
    ///
    /// This only returns a subslice spanning the unmodified
    /// file contents without decompressing them.
    pub fn extract<'a>(&self, raw_archive: &'a [u8]) -> &'a [u8] {
        let offset = self.offset as usize;
        let size = self.size();

        &raw_archive[offset..offset + size]
    }
}

/// Representation of a WAD archive.
///
/// This does not account for the dynamically-sized data
/// which follow after the structured archive start.
///
/// Implementations must consider that this does not parse
/// or represent the whole archive file and appropriately
/// work around this.
#[binread]
#[br(magic = b"KIWAD")]
pub struct Archive {
    /// The archive [`Header`].
    pub header: Header,
    /// The [`File`] metadata for all archived files.
    #[br(count = header.file_count)]
    pub files: Vec<File>,
}

impl Archive {
    /// Parses the archive from a given reader.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        reader.read_le().map_err(Into::into)
    }

    /// Verifies the CRC of every file in the archive given
    /// the raw archive bytes.
    pub fn verify_crcs(&self, raw_archive: &[u8]) -> anyhow::Result<()> {
        self.files.iter().try_for_each(|f| {
            let hash = crc::hash(f.extract(raw_archive));
            if hash == f.crc {
                Ok(())
            } else {
                bail!("CRC mismatch - expected {}, got {}", hash, f.crc);
            }
        })
    }
}
