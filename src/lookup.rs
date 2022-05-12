use std::ops::Range;

use indexmap::IndexSet;

const GLOBAL_SCOPE_SENTINEL: u32 = u32::MAX;
const ANONYMOUS_SCOPE_SENTINEL: u32 = u32::MAX - 1;

/// An Error that can happen when building a [`ScopeIndex`].
#[derive(Debug)]
pub struct ScopeIndexError(());

impl std::error::Error for ScopeIndexError {}

impl std::fmt::Display for ScopeIndexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("source could not be converted to source context")
    }
}

/// An indexed structure of Scopes that allows quick lookup.
#[derive(Debug)]
pub struct ScopeIndex {
    names: IndexSet<String>,
    /// Offset -> Index into `names` (or `u32::MAX` for `None`)
    ranges: Vec<(u32, u32)>,
}

impl ScopeIndex {
    /// Creates a new Scope index from the given list of Scopes.
    pub fn new(mut scopes: Vec<(Range<u32>, Option<String>)>) -> Result<Self, ScopeIndexError> {
        let mut names = IndexSet::new();
        let mut ranges = vec![];

        scopes.sort_by_key(|s| s.0.start);

        // TODO: resolve nesting and put in closing markers
        //let mut stack = vec![];

        for (range, name) in scopes {
            let name_idx = match name {
                Some(name) => names
                    .insert_full(name)
                    .0
                    .try_into()
                    .map_err(|_| ScopeIndexError(()))?,
                None => ANONYMOUS_SCOPE_SENTINEL,
            };

            ranges.push((range.start, name_idx));
        }

        Ok(Self { names, ranges })
    }

    /// Looks up the scope corresponding to the given `offset`.
    pub fn lookup(&self, offset: u32) -> ScopeLookupResult {
        let range_idx = match self.ranges.binary_search_by_key(&offset, |r| r.0) {
            Ok(idx) => idx,
            Err(0) => return ScopeLookupResult::Unknown,
            Err(idx) => idx - 1,
        };

        let name_idx = match self.ranges.get(range_idx) {
            Some(r) => r.1,
            None => return ScopeLookupResult::Unknown,
        };

        if name_idx == GLOBAL_SCOPE_SENTINEL {
            ScopeLookupResult::Unknown
        } else if name_idx == ANONYMOUS_SCOPE_SENTINEL {
            ScopeLookupResult::AnonymousScope
        } else {
            match self.names.get_index(name_idx as usize) {
                Some(name) => ScopeLookupResult::NamedScope(name.as_str()),
                None => ScopeLookupResult::Unknown,
            }
        }
    }
}

/// The Result of a Scope lookup.
pub enum ScopeLookupResult<'data> {
    /// A named function scope.
    NamedScope(&'data str),
    /// An anonymous function scope for which no name was inferred.
    AnonymousScope,
    /// The lookup did not result in any scope match.
    ///
    /// This most likely means that the offset belongs to the "global" scope.
    Unknown,
}
