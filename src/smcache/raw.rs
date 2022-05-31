/// The magic file preamble as individual bytes.
const SMCACHE_MAGIC_BYTES: [u8; 4] = *b"SYMC";

/// The magic file preamble to identify SymCache files.
///
/// Serialized as ASCII "SYMC" on little-endian (x64) systems.
pub const SMCACHE_MAGIC: u32 = u32::from_le_bytes(SMCACHE_MAGIC_BYTES);
/// The byte-flipped magic, which indicates an endianness mismatch.
pub const SMCACHE_MAGIC_FLIPPED: u32 = SMCACHE_MAGIC.swap_bytes();

/// The SmCache binary Header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Header {
    /// The file magic representing the file format and endianness.
    pub magic: u32,
    /// The Format Version.
    pub version: u32,

    /// The number of ranges covered by this file.
    pub ranges: u32,

    /// The total number of bytes in the string table.
    pub string_bytes: u32,

    /// Some reserved space in the header for future extensions that would not require a
    /// completely new parsing method.
    pub _reserved: [u8; 12],
}

/// A lookup source position of line/column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct SourcePosition {
    pub line: u32,
    pub column: u32,
}

/// A compressed Source Location.
///
/// This packs 21 bits of each of `file_idx`, `line`, `scope_idx` into a `u64`.
/// It looks a bit like this:
/// 0xxxxxxx xxxxxxxx xxxxxxxy yyyyyyyy yyyyyyyy yyyzzzzz zzzzzzzz zzzzzzzz
/// |^--- file_idx ---------^^------ line ---------^^------ scope_idx ----^
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct CompressedSourceLocation(u64);

/// Bit-shift used for [`CompressedSourceLocation`] contents.
pub const COMPRESSED_SHIFT: usize = 21;
/// Mask used for [`CompressedSourceLocation`] contents.
pub const COMPRESSED_MASK: u64 = (1 << COMPRESSED_SHIFT) - 1;
/// Sentinel value used to denote unknown file.
pub const NO_FILE_SENTINEL: u32 = (1 << COMPRESSED_SHIFT) - 1;
/// Sentinel value used to denote unknown/global scope.
pub const GLOBAL_SCOPE_SENTINEL: u32 = (1 << COMPRESSED_SHIFT) - 1;
/// Sentinel value used to denote anonymous function scope.
pub const ANONYMOUS_SCOPE_SENTINEL: u32 = (1 << COMPRESSED_SHIFT) - 2;

impl CompressedSourceLocation {
    pub fn new(sl: RawSourceLocation) -> Self {}
    pub fn unpack(self) -> RawSourceLocation {}
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RawSourceLocation {
    pub file_idx: u32,
    pub line: u32,
    pub scope_idx: u32,
}

/// Returns the amount left to add to the remainder to get 8 if
/// `to_align` isn't a multiple of 8.
pub fn align_to_eight(to_align: usize) -> usize {
    let remainder = to_align % 8;
    if remainder == 0 {
        remainder
    } else {
        8 - remainder
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn test_sizeof() {
        assert_eq!(mem::size_of::<Header>(), 24);
        assert_eq!(mem::align_of::<Header>(), 4);

        assert_eq!(mem::size_of::<SourcePosition>(), 8);
        assert_eq!(mem::align_of::<SourcePosition>(), 4);

        assert_eq!(mem::size_of::<CompressedSourceLocation>(), 8);
        assert_eq!(mem::align_of::<CompressedSourceLocation>(), 8);
    }
}
