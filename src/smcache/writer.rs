use std::collections::HashMap;
use std::io::Write;

use sourcemap::DecodedMap;

use crate::source::{SourceContext, SourceContextError};
use crate::{
    extract_scope_names, NameResolver, ScopeIndex, ScopeIndexError, ScopeLookupResult,
    SourcePosition,
};

use super::raw;
use raw::{ANONYMOUS_SCOPE_SENTINEL, GLOBAL_SCOPE_SENTINEL, NO_FILE_SENTINEL};

/// A structure that allows quick resolution of minified [`raw::SourcePosition`]s
/// to the original [`raw::SourceLocation`] it maps to.
pub struct SmCacheWriter {
    string_bytes: Vec<u8>,

    ranges: Vec<(SourcePosition, raw::SourceLocation)>,
}

impl SmCacheWriter {
    /// Constructs a new Cache from a minified source file and its corresponding SourceMap.
    pub fn new(source: &str, sourcemap: &str) -> Result<Self, SmCacheWriterError> {
        // TODO: we could sprinkle a few `tracing` spans around this function to
        // figure out which step is expensive and worth optimizing:
        //
        // - sourcemap parsing/flattening
        // - extracting scopes
        // - resolving scopes to original names
        // - converting the indexed scopes into line/col
        // - actually mapping all the tokens and collecting them into the cache
        //
        // what _might_ be expensive is resolving original scope names and
        // converting the `scope_index`, as the offset/position conversion is
        // potentially an `O(n)` operation due to UTF-16 :-(
        // so micro-optimizing that further _might_ be worth it.

        let sm =
            sourcemap::decode_slice(sourcemap.as_bytes()).map_err(SmCacheErrorInner::SourceMap)?;

        // flatten the `SourceMapIndex`, as we want to iterate tokens
        let sm = match sm {
            DecodedMap::Regular(sm) => DecodedMap::Regular(sm),
            DecodedMap::Index(smi) => {
                DecodedMap::Regular(smi.flatten().map_err(SmCacheErrorInner::SourceMap)?)
            }
            DecodedMap::Hermes(smh) => DecodedMap::Hermes(smh),
        };
        let tokens = match &sm {
            DecodedMap::Regular(sm) => sm.tokens(),
            DecodedMap::Hermes(smh) => smh.tokens(),
            DecodedMap::Index(_smi) => unreachable!(),
        };

        let scopes = extract_scope_names(source);

        // resolve scopes to original names
        let ctx = SourceContext::new(source).map_err(SmCacheErrorInner::SourceContext)?;
        let resolver = NameResolver::new(&ctx, &sm);
        let scopes: Vec<_> = scopes
            .into_iter()
            .map(|(range, name)| {
                let name = name.map(|n| resolver.resolve_name(&n));
                (range, name)
            })
            .collect();

        // convert our offset index to a source position index
        let scope_index = ScopeIndex::new(scopes).map_err(SmCacheErrorInner::ScopeIndex)?;
        let scope_index: Vec<_> = scope_index
            .iter()
            .filter_map(|(offset, result)| {
                let pos = ctx.offset_to_position(offset);
                pos.map(|pos| (pos, result))
            })
            .collect();
        let lookup_scope = |sp: &SourcePosition| {
            let idx = match scope_index.binary_search_by_key(&sp, |idx| &idx.0) {
                Ok(idx) => idx,
                Err(0) => 0,
                Err(idx) => idx - 1,
            };
            match scope_index.get(idx) {
                Some(r) => r.1,
                None => ScopeLookupResult::Unknown,
            }
        };

        // iterate over the tokens and create our index
        let mut string_bytes = Vec::new();
        let mut strings = HashMap::new();
        let mut ranges = Vec::new();

        let mut last = None;
        for token in tokens {
            let (min_line, min_col) = token.get_dst();
            let sp = SourcePosition::new(min_line, min_col);
            let file = token.get_source();
            let line = token.get_src_line();
            let scope = lookup_scope(&sp);

            let file_idx = match file {
                Some(file) => std::cmp::min(
                    Self::insert_string(&mut string_bytes, &mut strings, file),
                    NO_FILE_SENTINEL,
                ),
                None => NO_FILE_SENTINEL,
            };

            let scope_idx = match scope {
                ScopeLookupResult::NamedScope(name) => std::cmp::min(
                    Self::insert_string(&mut string_bytes, &mut strings, name),
                    GLOBAL_SCOPE_SENTINEL,
                ),
                ScopeLookupResult::AnonymousScope => ANONYMOUS_SCOPE_SENTINEL,
                ScopeLookupResult::Unknown => GLOBAL_SCOPE_SENTINEL,
            };

            let sl = raw::SourceLocation {
                file_idx,
                line,
                scope_idx,
            };

            if last == Some(sl) {
                continue;
            }
            ranges.push((sp, sl));
            last = Some(sl);
        }

        Ok(Self {
            string_bytes,
            ranges,
        })
    }

    /// Insert a string into this converter.
    ///
    /// If the string was already present, it is not added again. A newly added string
    /// is prefixed by its length as a `u32`. The returned `u32`
    /// is the offset into the `string_bytes` field where the string is saved.
    fn insert_string(
        string_bytes: &mut Vec<u8>,
        strings: &mut HashMap<String, u32>,
        s: &str,
    ) -> u32 {
        if s.is_empty() {
            return u32::MAX;
        }
        if let Some(&offset) = strings.get(s) {
            return offset;
        }
        let string_offset = string_bytes.len() as u32;
        let string_len = s.len() as u64;
        leb128::write::unsigned(string_bytes, string_len).unwrap();
        string_bytes.extend(s.bytes());

        strings.insert(s.to_owned(), string_offset);
        string_offset
    }

    /// Serialize the converted data.
    ///
    /// This writes the SymCache binary format into the given [`Write`].
    pub fn serialize<W: Write>(self, writer: &mut W) -> std::io::Result<()> {
        let mut writer = WriteWrapper::new(writer);

        let num_ranges = self.ranges.len() as u32;
        let string_bytes = self.string_bytes.len() as u32;

        let header = raw::Header {
            magic: raw::SMCACHE_MAGIC,
            version: raw::SMCACHE_VERSION,

            num_ranges,
            string_bytes,

            _reserved: [0; 12],
        };

        writer.write(&[header])?;
        writer.align()?;

        for (sp, _) in &self.ranges {
            let sp = raw::SourcePosition {
                line: sp.line,
                column: sp.column,
            };
            writer.write(&[sp])?;
        }
        writer.align()?;
        for (_, sl) in self.ranges {
            let compressed = raw::CompressedSourceLocation::new(sl);
            writer.write(&[compressed])?;
        }
        writer.align()?;

        writer.write(&self.string_bytes)?;

        Ok(())
    }
}

/// An Error that can happen when building a [`SmCache`].
#[derive(Debug)]
pub struct SmCacheWriterError(SmCacheErrorInner);

impl From<SmCacheErrorInner> for SmCacheWriterError {
    fn from(inner: SmCacheErrorInner) -> Self {
        SmCacheWriterError(inner)
    }
}

#[derive(Debug)]
enum SmCacheErrorInner {
    SourceMap(sourcemap::Error),
    ScopeIndex(ScopeIndexError),
    SourceContext(SourceContextError),
}

impl std::error::Error for SmCacheWriterError {}

impl std::fmt::Display for SmCacheWriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            SmCacheErrorInner::SourceMap(e) => e.fmt(f),
            SmCacheErrorInner::ScopeIndex(e) => e.fmt(f),
            SmCacheErrorInner::SourceContext(e) => e.fmt(f),
        }
    }
}

struct WriteWrapper<W> {
    writer: W,
    position: usize,
}

impl<W: Write> WriteWrapper<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            position: 0,
        }
    }

    fn write<T>(&mut self, data: &[T]) -> std::io::Result<usize> {
        let pointer = data.as_ptr() as *const u8;
        let len = std::mem::size_of_val(data);
        // SAFETY: both pointer and len are derived directly from data/T and are valid.
        let buf = unsafe { std::slice::from_raw_parts(pointer, len) };
        self.writer.write_all(buf)?;
        self.position += len;
        Ok(len)
    }

    fn align(&mut self) -> std::io::Result<usize> {
        let buf = &[0u8; 7];
        let len = raw::align_to_eight(self.position);
        self.write(&buf[0..len])
    }
}
