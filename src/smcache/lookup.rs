use std::mem;

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
    source_positions: &'data [raw::SourcePosition],
    source_locations: &'data [raw::CompressedSourceLocation],
    string_bytes: &'data [u8],
}

impl<'data> SmCache<'data> {
    pub fn parse(buf: &'data [u8]) -> Result<Self> {
        todo!()

        /*if align_to_eight(buf.as_ptr() as usize) != 0 {
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

        let mut files_size = mem::size_of::<raw::File>() * header.num_files as usize;
        files_size += align_to_eight(files_size);

        let mut functions_size = mem::size_of::<raw::Function>() * header.num_functions as usize;
        functions_size += align_to_eight(functions_size);

        let mut source_locations_size =
            mem::size_of::<raw::SourceLocation>() * header.num_source_locations as usize;
        source_locations_size += align_to_eight(source_locations_size);

        let mut ranges_size = mem::size_of::<raw::Range>() * header.num_ranges as usize;
        ranges_size += align_to_eight(ranges_size);

        let expected_buf_size = header_size
            + files_size
            + functions_size
            + source_locations_size
            + ranges_size
            + header.string_bytes as usize;

        if buf.len() < expected_buf_size || source_locations_size < ranges_size {
            return Err(Error::BadFormatLength);
        }

        // SAFETY: we just made sure that all the pointers we are constructing via pointer
        // arithmetic are within `buf`
        let files_start = unsafe { buf.as_ptr().add(header_size) };
        let functions_start = unsafe { files_start.add(files_size) };
        let source_locations_start = unsafe { functions_start.add(functions_size) };
        let ranges_start = unsafe { source_locations_start.add(source_locations_size) };
        let string_bytes_start = unsafe { ranges_start.add(ranges_size) };

        // SAFETY: the above buffer size check also made sure we are not going out of bounds
        // here
        let files = unsafe {
            &*ptr::slice_from_raw_parts(files_start as *const raw::File, header.num_files as usize)
        };
        let functions = unsafe {
            &*ptr::slice_from_raw_parts(
                functions_start as *const raw::Function,
                header.num_functions as usize,
            )
        };
        let source_locations = unsafe {
            &*ptr::slice_from_raw_parts(
                source_locations_start as *const raw::SourceLocation,
                header.num_source_locations as usize,
            )
        };
        let ranges = unsafe {
            &*ptr::slice_from_raw_parts(
                ranges_start as *const raw::Range,
                header.num_ranges as usize,
            )
        };
        let string_bytes = unsafe {
            &*ptr::slice_from_raw_parts(string_bytes_start, header.string_bytes as usize)
        };

        Ok(SymCache {
            header,
            files,
            functions,
            source_locations,
            ranges,
            string_bytes,
        })*/
    }

    /// Resolves a string reference to the pointed-to `&str` data.
    fn get_string(&self, offset: u32) -> Option<&'data str> {
        if offset == u32::MAX {
            return None;
        }
        let len_offset = offset as usize;
        let len_size = std::mem::size_of::<u32>();
        let len = u32::from_ne_bytes(
            self.string_bytes
                .get(len_offset..len_offset + len_size)?
                .try_into()
                .unwrap(),
        ) as usize;

        let start_offset = len_offset + len_size;
        let end_offset = start_offset + len;
        let bytes = self.string_bytes.get(start_offset..end_offset)?;

        std::str::from_utf8(bytes).ok()
    }

    /// Looks up a [`SourcePosition`] in the minified source and resolves it
    /// to the original [`SourceLocation`].
    pub fn lookup(&self, sp: SourcePosition) -> Option<SourceLocation> {
        todo!()

        /*let range_idx = match self.ranges.binary_search_by_key(&sp, |r| r.0) {
            Ok(idx) => idx,
            Err(0) => 0,
            Err(idx) => idx - 1,
        };

        let range = self.ranges.get(range_idx)?;

        let file = self
            .files
            .get_index(range.1.file_idx as usize)
            .map(|s| s.as_str());
        let line = range.1.line;
        let scope = self.resolve_scope(range.1.scope_idx);
        Some(SourceLocation { file, line, scope })*/
    }

    fn resolve_scope(&self, scope_idx: u32) -> ScopeLookupResult {
        todo!()

        /*if scope_idx == GLOBAL_SCOPE_SENTINEL {
            ScopeLookupResult::Unknown
        } else if scope_idx == ANONYMOUS_SCOPE_SENTINEL {
            ScopeLookupResult::AnonymousScope
        } else {
            match self.scopes.get_index(scope_idx as usize) {
                Some(name) => ScopeLookupResult::NamedScope(name.as_str()),
                None => ScopeLookupResult::Unknown,
            }
        }*/
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
