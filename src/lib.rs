#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::ops::Range;

mod name_resolver;
mod scope_index;
mod scope_name;
mod source;
mod swc;

pub use name_resolver::NameResolver;
pub use scope_index::{ScopeIndex, ScopeIndexError, ScopeLookupResult};
pub use scope_name::{NameComponent, ScopeName};
pub use source::{SourceContext, SourceContextError, SourcePosition};

/// The Scopes extracted from a piece of JS Code.
pub type Scopes = Vec<(Range<u32>, Option<ScopeName>)>;

/// Extracts function scopes from the given JS-like `src`.
///
/// The returned Vec includes the [`Range`] of the function scope, in byte offsets
/// inside the `src`, and the corresponding function name. `None` in this case
/// denotes a function scope for which no name could be inferred from the
/// surrounding code, which can mostly happen for anonymous or arrow functions
/// used as immediate callbacks.
///
/// The range includes the whole range of the function expression, including the
/// leading `function` keyword, function argument parentheses and trailing brace
/// in case there is one.
/// The returned vector does not have a guaranteed sorting order, and is
/// implementation dependent.
///
/// # Examples
///
/// ```
/// let src = "const arrowFnExpr = (a) => a; function namedFnDecl() {}";
/// //                arrowFnExpr -^------^  ^------namedFnDecl------^
/// let mut scopes: Vec<_> = js_source_scopes::extract_scope_names(src)
///     .into_iter()
///     .map(|res| {
///         let components = res.1.map(|n| n.components().map(|c| {
///             (c.text().to_string(), c.range())
///         }).collect::<Vec<_>>());
///         (res.0, components)
///     }).collect();
/// scopes.sort_by_key(|s| s.0.start);
///
/// let expected = vec![
///   (20..28, Some(vec![(String::from("arrowFnExpr"), Some(6..17))])),
///   (30..55, Some(vec![(String::from("namedFnDecl"),Some(39..50))])),
/// ];
/// assert_eq!(scopes, expected);
/// ```
#[tracing::instrument(level = "trace", skip_all)]
pub fn extract_scope_names(src: &str) -> Scopes {
    swc::parse_with_swc(src)
}
