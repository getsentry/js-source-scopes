use zerocopy::LayoutVerified;

use crate::{ScopeLookupResult, SourcePosition};

use super::raw::{self, LineOffset};

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
    files: &'data [raw::File],
    line_offsets: &'data [raw::LineOffset],
    string_bytes: &'data [u8],
}

impl<'data> std::fmt::Debug for SmCache<'data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmCache")
            .field("version", &self.header.version)
            .field("mappings", &self.header.num_mappings)
            .field("files", &self.header.num_files)
            .field("line_offsets", &self.header.num_line_offsets)
            .field("string_bytes", &self.header.string_bytes)
            .finish()
    }
}

impl<'data> SmCache<'data> {
    pub fn parse(buf: &'data [u8]) -> Result<Self> {
        let (header, buf): (LayoutVerified<_, raw::Header>, _) =
            LayoutVerified::new_from_prefix(buf).ok_or(Error::Header)?;
        let header = header.into_ref();
        let buf = align_buf(buf);

        if header.magic == raw::SMCACHE_MAGIC_FLIPPED {
            return Err(Error::WrongEndianness);
        }
        if header.magic != raw::SMCACHE_MAGIC {
            return Err(Error::WrongFormat);
        }
        if header.version != raw::SMCACHE_VERSION {
            return Err(Error::WrongVersion);
        }

        let num_mappings = header.num_mappings as usize;
        let (min_source_positions, buf) = LayoutVerified::new_slice_from_prefix(buf, num_mappings)
            .ok_or(Error::SourcePositions)?;
        let min_source_positions = min_source_positions.into_slice();
        let buf = align_buf(buf);

        let (orig_source_locations, buf) = LayoutVerified::new_slice_from_prefix(buf, num_mappings)
            .ok_or(Error::SourceLocations)?;
        let orig_source_locations = orig_source_locations.into_slice();
        let buf = align_buf(buf);

        let (files, buf) = LayoutVerified::new_slice_from_prefix(buf, header.num_files as usize)
            .ok_or(Error::SourceLocations)?;
        let files = files.into_slice();
        let buf = align_buf(buf);

        let (line_offsets, buf) =
            LayoutVerified::new_slice_from_prefix(buf, header.num_line_offsets as usize)
                .ok_or(Error::SourceLocations)?;
        let line_offsets = line_offsets.into_slice();
        let buf = align_buf(buf);

        let string_bytes = header.string_bytes as usize;
        let string_bytes = buf.get(..string_bytes).ok_or(Error::StringBytes)?;

        Ok(Self {
            header,
            min_source_positions,
            orig_source_locations,
            files,
            line_offsets,
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

    /// Returns the [`File`] which allows fast access to source lines.
    pub fn get_file(&self, name: &str) -> Option<File> {
        let file_idx = self
            .files
            .binary_search_by_key(&name, |file| {
                // TODO: decoding the string here might be expensive.
                // however, doing that ahead of time when loading the file is
                // expensive too, so this is a tradeoff we could potentially measure.
                self.get_string(file.name_offset).unwrap_or("")
            })
            .ok()?;
        let file = self.files.get(file_idx)?;

        let source = self.get_string(file.source_offset)?;
        let line_offsets = self
            .line_offsets
            .get(file.line_offsets_start as usize..file.line_offsets_end as usize)?;

        Some(File {
            source,
            line_offsets,
        })
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The file was generated by a system with different endianness.
    //#[error("endianness mismatch")]
    WrongEndianness,
    /// The file magic does not match.
    //#[error("wrong format magic")]
    WrongFormat,
    /// The format version in the header is wrong/unknown.
    //#[error("unknown SymCache version")]
    WrongVersion,
    Header,
    SourcePositions,
    SourceLocations,
    StringBytes,
}

pub struct File<'data> {
    source: &'data str,
    line_offsets: &'data [LineOffset],
}

impl<'data> File<'data> {
    /// Returns the source of this file.
    pub fn get_source(&self) -> &str {
        self.source
    }

    /// Returns the requested source line if possible.
    pub fn get_line(&self, line_no: usize) -> Option<&str> {
        let from = self.line_offsets.get(line_no).copied()?.0 as usize;
        let to = self.line_offsets.get(line_no.checked_add(1)?).copied()?.0 as usize;
        self.source.get(from..to)
    }
}

fn align_buf(buf: &[u8]) -> &[u8] {
    let offset = buf.as_ptr().align_offset(8);
    buf.get(offset..).unwrap_or(&[])
}
