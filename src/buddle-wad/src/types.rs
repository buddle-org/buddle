//! Common types and structures in the KIWAD format.

use anyhow::bail;
use binrw::{
    binread,
    io::{Read, Seek, SeekFrom},
    BinRead, BinReaderExt, BinResult, Endian, VecArgs,
};

use crate::crc;

/// The header of a KIWAD archive.
#[binread]
pub struct Header {
    /// The format version in use.
    pub version: u32,
    /// The total number of files stored in the archive.
    pub file_count: u32,
    /// The configuration flags associated with the archive.
    ///
    /// These are only present when [`Header::version`] is `2`
    /// or greater.
    #[br(if(version >= 2))]
    pub flags: Option<u8>,
}

/// Metadata for a file stored in an archive.
#[binread]
pub struct File {
    /// The starting offset of the file datta.
    pub offset: u32,
    /// The uncompressed size of the file contents.
    pub uncompressed_size: u32,
    /// The compressed size of the file contents.
    ///
    /// When the file is stored uncompressed, this should be ignored.
    pub compressed_size: u32,
    /// Whether the file is stored compressed.
    #[br(map = |x: u8| x != 0)]
    pub compressed: bool,
    /// The CRC32 checksum of uncompressed file contents.
    pub crc: u32,
    // Length of the following name; only stored temporarily for reading.
    #[br(temp)]
    name_len: u32,
    /// The name of the file in the archive.
    #[br(args(name_len as usize), parse_with = parse_file_name)]
    pub name: String,
}

impl File {
    /// Gets the length of data described by this file in bytes.
    #[inline]
    pub const fn size(&self) -> usize {
        if self.compressed {
            self.compressed_size as usize
        } else {
            self.uncompressed_size as usize
        }
    }

    /// Extracts this file from the given raw archive bytes.
    ///
    /// # Panics
    ///
    /// This may panic when `raw_archive` is indexed incorrectly with
    /// offset and length of the described file bytes.
    pub fn extract<'wad>(&self, raw_archive: &'wad [u8]) -> &'wad [u8] {
        let offset = self.offset as usize;
        let size = self.size();

        &raw_archive[offset..offset + size]
    }
}

/// Representation of a KIWAD archive.
///
/// This does not account for the dynamically-sized data which
/// follows after the structured archive start.
///
/// Implementations must consider this and keep the raw archive
/// bytes around even after parsing this structure.
#[binread]
#[br(magic = b"KIWAD")]
pub struct Archive {
    /// The archive [`Header`].
    pub header: Header,
    /// [`File`] metadata describing every stored file.
    #[br(count = header.file_count)]
    pub files: Vec<File>,
}

impl Archive {
    /// Parses the archive from the given [`Read`]er.
    pub fn parse<R: Read + Seek>(reader: &mut R) -> anyhow::Result<Self> {
        reader.read_le().map_err(Into::into)
    }

    /// Verifies the CRCs of every file in the archive given the
    /// raw bytes of the archive file.
    pub fn verify_crcs(&self, raw_archive: &[u8]) -> anyhow::Result<()> {
        self.files.iter().try_for_each(|f| {
            let hash = crc::hash(f.extract(raw_archive));
            if hash == f.crc {
                Ok(())
            } else {
                bail!("CRC mismatch - expected {hash}, got {}", f.crc)
            }
        })
    }
}

#[inline]
fn parse_file_name<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    (len,): (usize,),
) -> BinResult<String> {
    // Read all string bytes and chop off the null terminator byte.
    let out = Vec::<u8>::read_options(
        reader,
        endian,
        VecArgs::builder().count(len.saturating_sub(1)).finalize(),
    )?;
    let new_pos = reader.seek(SeekFrom::Current(1))?;

    String::from_utf8(out).map_err(|e| binrw::Error::Custom {
        pos: new_pos - len as u64,
        err: Box::new(e.utf8_error()),
    })
}
