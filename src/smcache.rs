use indexmap::IndexSet;
use sourcemap::DecodedMap;

use crate::lookup::{ScopeLookupResult, ANONYMOUS_SCOPE_SENTINEL, GLOBAL_SCOPE_SENTINEL};
use crate::source::{SourceContext, SourceContextError, SourcePosition};
use crate::{extract_scope_names, NameResolver, ScopeIndex, ScopeIndexError};

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

#[derive(Clone, Copy, Debug, PartialEq)]
struct RawSourceLocation {
    file_idx: u32,
    line: u32,
    scope_idx: u32,
}

const NO_FILE_SENTINEL: u32 = u32::MAX;

/// A structure that allows quick resolution of minified [`SourcePosition`]s
/// to the original [`SourceLocation`] it maps to.
pub struct SmCache {
    files: IndexSet<String>,
    scopes: IndexSet<String>,
    ranges: Vec<(SourcePosition, RawSourceLocation)>,
}

impl SmCache {
    /// Constructs a new Cache from a minified source file and its corresponding SourceMap.
    pub fn new(source: &str, sourcemap: &str) -> Result<Self, SmCacheError> {
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
        let mut files = IndexSet::new();
        let mut scopes = IndexSet::new();
        let mut ranges = Vec::new();

        let mut last = None;
        for token in tokens {
            let (min_line, min_col) = token.get_dst();
            let sp = SourcePosition::new(min_line, min_col);
            let file = token.get_source();
            let line = token.get_src_line();
            let scope = lookup_scope(&sp);

            let file_idx = match file {
                Some(file) => match files.get_index_of(file) {
                    Some(idx) => idx as u32,
                    None => files.insert_full(file.to_owned()).0 as u32,
                },
                None => NO_FILE_SENTINEL,
            };

            let scope_idx = match scope {
                ScopeLookupResult::NamedScope(name) => match scopes.get_index_of(name) {
                    Some(idx) => idx as u32,
                    None => scopes.insert_full(name.to_owned()).0 as u32,
                },
                ScopeLookupResult::AnonymousScope => ANONYMOUS_SCOPE_SENTINEL,
                ScopeLookupResult::Unknown => GLOBAL_SCOPE_SENTINEL,
            };

            let sl = RawSourceLocation {
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
            files,
            scopes,
            ranges,
        })
    }

    /// Looks up a [`SourcePosition`] in the minified source and resolves it
    /// to the original [`SourceLocation`].
    pub fn lookup(&self, sp: SourcePosition) -> Option<SourceLocation> {
        let range_idx = match self.ranges.binary_search_by_key(&sp, |r| r.0) {
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
        Some(SourceLocation { file, line, scope })
    }

    fn resolve_scope(&self, scope_idx: u32) -> ScopeLookupResult {
        if scope_idx == GLOBAL_SCOPE_SENTINEL {
            ScopeLookupResult::Unknown
        } else if scope_idx == ANONYMOUS_SCOPE_SENTINEL {
            ScopeLookupResult::AnonymousScope
        } else {
            match self.scopes.get_index(scope_idx as usize) {
                Some(name) => ScopeLookupResult::NamedScope(name.as_str()),
                None => ScopeLookupResult::Unknown,
            }
        }
    }
}

/// An Error that can happen when building a [`SmCache`].
#[derive(Debug)]
pub struct SmCacheError(SmCacheErrorInner);

impl From<SmCacheErrorInner> for SmCacheError {
    fn from(inner: SmCacheErrorInner) -> Self {
        SmCacheError(inner)
    }
}

#[derive(Debug)]
enum SmCacheErrorInner {
    SourceMap(sourcemap::Error),
    ScopeIndex(ScopeIndexError),
    SourceContext(SourceContextError),
}

impl std::error::Error for SmCacheError {}

impl std::fmt::Display for SmCacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            SmCacheErrorInner::SourceMap(e) => e.fmt(f),
            SmCacheErrorInner::ScopeIndex(e) => e.fmt(f),
            SmCacheErrorInner::SourceContext(e) => e.fmt(f),
        }
    }
}
