//! Common types in the KIWAD format.

use binrw::{
    binread,
    io::{Read, Seek},
    BinReaderExt, BinResult,
};

use crate::parse::parse_file_name;

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
    pub fn parse<R: Read + Seek>(reader: &mut R) -> BinResult<Self> {
        reader.read_le()
    }
}
