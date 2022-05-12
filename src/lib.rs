use std::ops::Range;

mod lookup;
mod rslint;
mod source;

pub use lookup::{ScopeIndex, ScopeIndexError, ScopeLookupResult};
pub use source::{SourceContext, SourceContextError, SourcePosition};

/// Extracts function scopes from the given JS-like `src`.
///
/// The returned Vec includes the `Range` of the function scope, in byte offsets
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
/// let mut scopes = js_source_scopes::extract_scope_names(src);
/// scopes.sort_by_key(|s| s.0.start);
///
/// let expected = vec![
///   (20..28, Some(String::from("arrowFnExpr"))),
///   (30..55, Some(String::from("namedFnDecl"))),
/// ];
/// assert_eq!(scopes, expected);
/// ```
pub fn extract_scope_names(src: &str) -> Vec<(Range<u32>, Option<String>)> {
    rslint::parse_with_rslint(src)
}

/*mod swc {
    use swc_ecma_parser::lexer::Lexer;
    use swc_ecma_parser::{Parser, StringInput, TsConfig};

    pub fn parse_with_swc(src: &str) {
        swc_ecma_parser::parse_file_as_module();

        let source = SourceFile;

        let mut parser = Parser::new(
            swc_ecma_parser::Syntax::Typescript(TsConfig {
                tsx: true,
                decorators: true,
                dts: true,
                no_early_errors: true,
            }),
            StringInput::from(src),
            None,
        );

        let module = parser.parse_module().unwrap();
    }
}*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_correct_scopes() {
        let src = std::fs::read_to_string("tests/fixtures/trace/sync.mjs").unwrap();

        let scopes = extract_scope_names(&src);
        dbg!(scopes);

        let ctx = SourceContext::new(&src).unwrap();

        // node gives the following stacktrace for the above file:
        // at Object.objectLiteralAnon (.../sync.mjs:84:11)
        // at Object.objectLiteralMethod (.../sync.mjs:81:9)
        // at localReassign (.../sync.mjs:76:7)
        // at Klass.prototypeMethod (.../sync.mjs:71:28)
        // at Klass.#privateMethod (.../sync.mjs:40:10)
        // at Klass.classCallbackArrow (.../sync.mjs:36:24)
        // at .../sync.mjs:65:34
        // at callsSyncCallback (.../shared.mjs:2:3)
        // at Klass.classCallbackBound (.../sync.mjs:65:5)
        // at callsSyncCallback (.../shared.mjs:2:3)
        // at Klass.classCallbackSelf (.../sync.mjs:61:5)
        // at .../sync.mjs:56:12
        // at callsSyncCallback (.../shared.mjs:2:3)
        // at Klass.classMethod (.../sync.mjs:55:5)
        // at new BaseKlass (.../sync.mjs:32:10)
        // at new Klass (.../sync.mjs:50:5)
        // at Function.staticMethod (.../sync.mjs:46:5)
        // at .../sync.mjs:22:17
        // at callsSyncCallback (.../shared.mjs:2:3)
        // at .../sync.mjs:21:9
        // at callsSyncCallback (.../shared.mjs:2:3)
        // at namedImmediateCallback (.../sync.mjs:19:7)
        // at callsSyncCallback (.../shared.mjs:2:3)
        // at namedDeclaredCallback (.../sync.mjs:17:5)
        // at callsSyncCallback (.../shared.mjs:2:3)
        // at arrowFn (.../sync.mjs:27:3)
        // at anonFn (.../sync.mjs:12:3)
        // at namedFnExpr (.../sync.mjs:8:3)
        // at namedFn (.../sync.mjs:4:3)

        // NOTE: all the source positions in the stack trace are 1-based
        // `localReassign`:
        dbg!(ctx.position_to_offset(SourcePosition::new(75, 6)));
        // `namedImmediateCallback`:
        dbg!(ctx.position_to_offset(SourcePosition::new(18, 6)));
        // `namedDeclaredCallback`:
        dbg!(ctx.position_to_offset(SourcePosition::new(16, 4)));
        // `arrowFn`:
        dbg!(ctx.position_to_offset(SourcePosition::new(26, 2)));
    }
}
