// TODO: should we rather have `usize` everywhere instead of `u32`?

use std::ops::Range;

mod lookup;
mod rslint;
mod scope_name;
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
        .into_iter()
        .map(|res| (res.0, res.1.map(|s| s.to_string())))
        .collect()
}

// TODO: maybe see if swc makes scope extraction easier / faster ?
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
