use sourcemap::DecodedMap;

use crate::lookup::{ScopeLookupResult, ANONYMOUS_SCOPE_SENTINEL, GLOBAL_SCOPE_SENTINEL};
use crate::source::{SourceContext, SourceContextError, SourcePosition};
use crate::{extract_scope_names, NameResolver, ScopeIndex, ScopeIndexError};

mod lookup;
mod raw;
mod writer;
