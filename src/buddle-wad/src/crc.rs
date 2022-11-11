//! CRC32 calculation for integrity checking uncompressed
//! files in archives.

use crc32fast::Hasher;

/// Computes the CRC32 of `data`, as encoded in WAD archives.
pub fn hash(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new_with_initial(u32::MAX);
    hasher.update(data);
    hasher.finalize() ^ u32::MAX
}
