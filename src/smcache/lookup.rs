use std::{mem, ptr};

use crate::{ScopeLookupResult, SourcePosition};

use super::raw;

use raw::align_to_eight;

/// A resolved Source Location  with file, line and scope information.
#[derive(Debug, PartialEq)]
pub struct SourceLocation<'data> {
    /// The source file this location belongs to.
    pub file: Option<&'data str>,
    /// The source line.
    pub line: u32,
    /// The scope containing this source location.
    pub scope: ScopeLookupResult<'data>,
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub struct SmCache<'data> {
    header: &'data raw::Header,
    source_positions: &'data [(raw::SourcePosition, raw::CompressedSourceLocation)],
    string_bytes: &'data [u8],
}

impl<'data> std::fmt::Debug for SmCache<'data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmCache")
            .field("version", &self.header.version)
            .field("ranges", &self.header.num_ranges)
            .field("string_bytes", &self.header.string_bytes)
            .finish()
    }
}

impl<'data> SmCache<'data> {
    pub fn parse(buf: &'data [u8]) -> Result<Self> {
        if align_to_eight(buf.as_ptr() as usize) != 0 {
            return Err(Error::BufferNotAligned);
        }

        let mut header_size = mem::size_of::<raw::Header>();
        header_size += align_to_eight(header_size);

        if buf.len() < header_size {
            return Err(Error::HeaderTooSmall);
        }
        // SAFETY: we checked that the buffer is well aligned and large enough to fit a `raw::Header`.
        let header = unsafe { &*(buf.as_ptr() as *const raw::Header) };
        if header.magic == raw::SMCACHE_MAGIC_FLIPPED {
            return Err(Error::WrongEndianness);
        }
        if header.magic != raw::SMCACHE_MAGIC {
            return Err(Error::WrongFormat);
        }
        if header.version != raw::SMCACHE_VERSION {
            return Err(Error::WrongVersion);
        }

        let mut source_positions_size =
            mem::size_of::<(raw::SourcePosition, raw::CompressedSourceLocation)>()
                * header.num_ranges as usize;
        source_positions_size += align_to_eight(source_positions_size);

        let expected_buf_size = header_size + source_positions_size + header.string_bytes as usize;

        if buf.len() < expected_buf_size {
            return Err(Error::BadFormatLength);
        }

        // SAFETY: we just made sure that all the pointers we are constructing via pointer
        // arithmetic are within `buf`
        let source_positions_start = unsafe { buf.as_ptr().add(header_size) };
        let string_bytes_start = unsafe { source_positions_start.add(source_positions_size) };

        // SAFETY: the above buffer size check also made sure we are not going out of bounds
        // here
        let source_positions = unsafe {
            &*ptr::slice_from_raw_parts(
                source_positions_start
                    as *const (raw::SourcePosition, raw::CompressedSourceLocation),
                header.num_ranges as usize,
            )
        };
        let string_bytes = unsafe {
            &*ptr::slice_from_raw_parts(string_bytes_start, header.string_bytes as usize)
        };

        Ok(SmCache {
            header,
            source_positions,
            string_bytes,
        })
    }

    /// Resolves a string reference to the pointed-to `&str` data.
    fn get_string(&self, offset: u32) -> Option<&'data str> {
        if offset == u32::MAX {
            return None;
        }
        let len_offset = offset as usize;
        let reader = &mut &self.string_bytes[len_offset..];
        let len = leb128::read::unsigned(reader).ok()? as usize;

        let bytes = reader.get(..len)?;

        std::str::from_utf8(bytes).ok()
    }

    /// Looks up a [`SourcePosition`] in the minified source and resolves it
    /// to the original [`SourceLocation`].
    pub fn lookup(&self, sp: SourcePosition) -> Option<SourceLocation> {
        let idx = match self
            .source_positions
            .binary_search_by_key(&sp.into(), |r| r.0)
        {
            Ok(idx) => idx,
            Err(0) => 0,
            Err(idx) => idx - 1,
        };

        let compressed = self.source_positions.get(idx)?.1;
        let unpacked = compressed.unpack();

        let file = self.get_string(unpacked.file_idx);
        let line = unpacked.line;

        let scope = match unpacked.scope_idx {
            raw::GLOBAL_SCOPE_SENTINEL => ScopeLookupResult::Unknown,
            raw::ANONYMOUS_SCOPE_SENTINEL => ScopeLookupResult::AnonymousScope,
            _ => self
                .get_string(unpacked.scope_idx)
                .map_or(ScopeLookupResult::Unknown, ScopeLookupResult::NamedScope),
        };

        Some(SourceLocation { file, line, scope })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The buffer is not correctly aligned.
    //#[error("source buffer is not correctly aligned")]
    BufferNotAligned,
    /// The header's size doesn't match our expected size.
    //#[error("header is too small")]
    HeaderTooSmall,
    /// The file was generated by a system with different endianness.
    //#[error("endianness mismatch")]
    WrongEndianness,
    /// The file magic does not match.
    //#[error("wrong format magic")]
    WrongFormat,
    /// The format version in the header is wrong/unknown.
    //#[error("unknown SymCache version")]
    WrongVersion,
    /// The self-advertised size of the buffer is not correct.
    //#[error("incorrect buffer length")]
    BadFormatLength,
}
