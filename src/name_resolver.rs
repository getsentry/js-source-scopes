use sourcemap::DecodedMap;

use crate::{NameComponent, ScopeName, SourceContext};

/// A structure for resolving [`ScopeName`]s in minified code to their original names
/// using information contained in a [`DecodedMap`].
pub struct NameResolver<'a, T> {
    ctx: &'a SourceContext<T>,
    sourcemap: &'a DecodedMap,
}

impl<'a, T: AsRef<str>> NameResolver<'a, T> {
    /// Construct a new [`NameResolver`] from a [`SourceContext`] (for the minified source) and a [`DecodedMap`].
    pub fn new(ctx: &'a SourceContext<T>, sourcemap: &'a DecodedMap) -> Self {
        Self { ctx, sourcemap }
    }

    /// Resolves the given minified [`ScopeName`] to the original name.
    ///
    /// This tries to resolve each [`NameComponent`] by looking up its source
    /// range in the [`DecodedMap`], using the token's `name` (as defined in the
    /// sourcemap `names`) when possible.
    pub fn resolve_name(&self, name: &ScopeName) -> String {
        name.components()
            .map(|c| self.try_map_token(c).unwrap_or_else(|| c.text()))
            .collect::<String>()
    }

    fn try_map_token(&self, c: &NameComponent) -> Option<&str> {
        let range = c.range()?;
        let source_position = self.ctx.offset_to_position(range.start)?;
        let token = self
            .sourcemap
            .lookup_token(source_position.line, source_position.column)?;

        let is_exactish_match = token.get_dst_line() == source_position.line
            && token.get_dst_col() >= source_position.column.saturating_sub(1);

        if is_exactish_match {
            token.get_name()
        } else {
            None
        }
    }
}
