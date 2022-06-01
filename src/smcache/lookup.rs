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
    min_source_positions: &'data [raw::MinifiedSourcePosition],
    orig_source_locations: &'data [raw::OriginalSourceLocation],
    string_bytes: &'data [u8],
}

impl<'data> std::fmt::Debug for SmCache<'data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmCache")
            .field("version", &self.header.version)
            .field("mappings", &self.header.num_mappings)
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

        let mut min_source_positions_size =
            mem::size_of::<raw::MinifiedSourcePosition>() * header.num_mappings as usize;
        min_source_positions_size += align_to_eight(min_source_positions_size);

        let mut orig_source_locations_size =
            mem::size_of::<raw::OriginalSourceLocation>() * header.num_mappings as usize;
        orig_source_locations_size += align_to_eight(orig_source_locations_size);

        let expected_buf_size = header_size
            + min_source_positions_size
            + orig_source_locations_size
            + header.string_bytes as usize;

        if buf.len() < expected_buf_size {
            return Err(Error::BadFormatLength);
        }

        // SAFETY: we just made sure that all the pointers we are constructing via pointer
        // arithmetic are within `buf`
        let min_source_positions_start = unsafe { buf.as_ptr().add(header_size) };
        let orig_source_locations_start =
            unsafe { min_source_positions_start.add(min_source_positions_size) };
        let string_bytes_start =
            unsafe { orig_source_locations_start.add(orig_source_locations_size) };

        // SAFETY: the above buffer size check also made sure we are not going out of bounds
        // here
        let min_source_positions = unsafe {
            &*ptr::slice_from_raw_parts(
                min_source_positions_start as *const raw::MinifiedSourcePosition,
                header.num_mappings as usize,
            )
        };
        let orig_source_locations = unsafe {
            &*ptr::slice_from_raw_parts(
                orig_source_locations_start as *const raw::OriginalSourceLocation,
                header.num_mappings as usize,
            )
        };
        let string_bytes = unsafe {
            &*ptr::slice_from_raw_parts(string_bytes_start, header.string_bytes as usize)
        };

        Ok(SmCache {
            header,
            min_source_positions,
            orig_source_locations,
            string_bytes,
        })
    }

    /// Resolves a string reference to the pointed-to `&str` data.
    fn get_string(&self, offset: u32) -> Option<&'data str> {
        let reader = &mut self.string_bytes.get(offset as usize..)?;
        let len = leb128::read::unsigned(reader).ok()? as usize;

        let bytes = reader.get(..len)?;

        std::str::from_utf8(bytes).ok()
    }

    /// Looks up a [`SourcePosition`] in the minified source and resolves it
    /// to the original [`SourceLocation`].
    pub fn lookup(&self, sp: SourcePosition) -> Option<SourceLocation> {
        let idx = match self.min_source_positions.binary_search(&sp.into()) {
            Ok(idx) => idx,
            Err(0) => 0,
            Err(idx) => idx - 1,
        };

        let sl = self.orig_source_locations.get(idx)?;

        let file = self.get_string(sl.file_idx);
        let line = sl.line;

        let scope = match sl.scope_idx {
            raw::GLOBAL_SCOPE_SENTINEL => ScopeLookupResult::Unknown,
            raw::ANONYMOUS_SCOPE_SENTINEL => ScopeLookupResult::AnonymousScope,
            idx => self
                .get_string(idx)
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
