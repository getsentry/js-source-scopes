use std::ops::Range;

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

/// A line/column source position.
#[derive(Debug, PartialEq, PartialOrd)]
pub struct SourcePosition {
    /// Line in the source file, 0-based.
    line: u32,
    /// Column in the source file, 0-based.
    ///
    /// The column is given in UTF-16 code points.
    column: u32,
}

impl SourcePosition {
    /// Create a new SourcePosition with the given line/column.
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

/// A Source Context allowing fast access to lines and line/column <-> byte offset remapping.
pub struct SourceContext<T> {
    src: T,
    line_offsets: Vec<u32>,
}

/// An Error that can happen when building a [`SourceContext`].
#[derive(Debug)]
pub struct SourceContextError(());

impl<T: AsRef<str>> SourceContext<T> {
    /// Construct a new Source Context from the given `src` buffer.
    pub fn new(src: T) -> Result<Self, SourceContextError> {
        let buf = src.as_ref();
        let buf_ptr = buf.as_ptr();
        let mut line_offsets: Vec<u32> = buf
            .lines()
            .map(|line| unsafe { line.as_ptr().offset_from(buf_ptr) as usize }.try_into())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| SourceContextError(()))?;
        line_offsets.push(buf.len().try_into().map_err(|_| SourceContextError(()))?);
        Ok(Self { src, line_offsets })
    }

    /// Get the `nth` line of the source, 0-based.
    pub fn get_line(&self, nth: u32) -> Option<&str> {
        let from = self.line_offsets.get(nth as usize).copied()? as usize;
        let to = self.line_offsets.get(nth as usize + 1).copied()? as usize;
        self.src.as_ref().get(from..to)
    }

    /// Converts a byte offset into the source to the corresponding line/column.
    ///
    /// The column is given in UTF-16 code points.
    pub fn offset_to_position(&self, offset: u32) -> Option<SourcePosition> {
        // `into_ok_or_err` is still nightly-only
        // TODO: fix this stuff, its so confusing, lol
        let line_no = match self.line_offsets.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line,
        };
        if line_no >= self.line_offsets.len() - 1 {
            return None;
        }

        let mut byte_offset = self.line_offsets.get(line_no).copied()? as usize;

        let line_no = line_no.try_into().ok()?;

        if byte_offset == offset as usize {
            return Some(SourcePosition::new(line_no, 0));
        }

        let line = self.get_line(line_no)?;

        let mut utf16_offset = 0;
        for c in line.chars() {
            utf16_offset += c.len_utf16();
            byte_offset += c.len_utf8();

            if byte_offset >= offset as usize {
                return Some(SourcePosition::new(line_no, utf16_offset.try_into().ok()?));
            }
        }

        None
    }

    /// Converts the given line/column to the corresponding byte offset inside the source.
    pub fn position_to_offset(&self, position: SourcePosition) -> Option<u32> {
        let SourcePosition { line, column } = position;

        let from = self.line_offsets.get(line as usize).copied()?;
        let to = self.line_offsets.get(line as usize + 1).copied()? as usize;

        if column == 0 {
            return Some(from);
        }
        let from = from as usize;

        let line = self.src.as_ref().get(from..to)?;

        let mut byte_offset = from;
        let mut utf16_offset = 0;
        let column = column as usize;
        for c in line.chars() {
            utf16_offset += c.len_utf16();
            byte_offset += c.len_utf8();

            if utf16_offset >= column {
                return byte_offset.try_into().ok();
            }
        }

        None
    }
}

mod rslint {
    use std::ops::Range;

    use rslint_parser::{ast, AstNode, SyntaxKind, SyntaxNode, SyntaxNodeExt, TextRange};

    fn convert_text_range(range: TextRange) -> Range<u32> {
        range.start().into()..range.end().into()
    }

    pub fn parse_with_rslint(src: &str) -> Vec<(Range<u32>, Option<String>)> {
        let parse =
            //rslint_parser::parse_with_syntax(src, 0, rslint_parser::FileKind::TypeScript.into());
            rslint_parser::parse_text(src, 0);

        let syntax = parse.syntax();
        //dbg!(&syntax);

        let mut ranges = vec![];

        for node in syntax.descendants() {
            if let Some(fn_decl) = node.try_to::<ast::FnDecl>() {
                let name = fn_decl
                    .name()
                    .map(|n| n.text())
                    .or_else(|| find_fn_name_from_ctx(&node));

                ranges.push((convert_text_range(node.text_range()), name));
            } else if let Some(fn_expr) = node.try_to::<ast::FnExpr>() {
                let name = fn_expr
                    .name()
                    .map(|n| n.text())
                    .or_else(|| find_fn_name_from_ctx(&node));

                ranges.push((convert_text_range(node.text_range()), name));
            } else if node.is::<ast::ArrowExpr>() {
                let name = find_fn_name_from_ctx(&node);

                ranges.push((convert_text_range(node.text_range()), name));
            }

            // TODO: method, constructor

            /*match node.kind() {
                SyntaxKind::METHOD => todo!(),
                SyntaxKind::CONSTRUCTOR => todo!(),
                _ => todo!(),
            }*/
        }

        ranges
    }

    fn find_fn_name_from_ctx(node: &SyntaxNode) -> Option<String> {
        // the node itself is the first "ancestor"
        for parent in node.ancestors().skip(1) {
            // break on syntax that itself starts a scope
            match parent.kind() {
                SyntaxKind::FN_DECL
                | SyntaxKind::FN_EXPR
                | SyntaxKind::ARROW_EXPR
                | SyntaxKind::METHOD
                | SyntaxKind::CONSTRUCTOR => return None,
                _ => {}
            }
            if let Some(assign_expr) = parent.try_to::<ast::AssignExpr>() {
                if let Some(ast::PatternOrExpr::Expr(expr)) = assign_expr.lhs() {
                    return Some(text_of_node(expr.syntax()));
                }
            }
            if let Some(decl) = parent.try_to::<ast::Declarator>() {
                if let Some(ast::Pattern::SinglePattern(sp)) = decl.pattern() {
                    return sp.name().map(|n| n.text());
                }
            }
        }
        None
    }

    fn text_of_node(node: &SyntaxNode) -> String {
        use std::fmt::Write;

        let mut s = String::new();
        for node in node
            .descendants_with_tokens()
            .filter_map(|x| x.into_token().filter(|token| !token.kind().is_trivia()))
        {
            let _ = write!(&mut s, "{}", node.text());
        }

        s
    }
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
    fn source_context() {
        let ctx = SourceContext::new("").unwrap();
        assert_eq!(ctx.get_line(0), None);
        assert_eq!(ctx.offset_to_position(0), None);
        assert_eq!(ctx.position_to_offset(SourcePosition::new(0, 0)), None);

        let src = "\n \r\na";
        let ctx = SourceContext::new(src).unwrap();
        assert_eq!(ctx.get_line(0), Some("\n"));
        assert_eq!(ctx.get_line(1), Some(" \r\n"));
        assert_eq!(ctx.get_line(2), Some("a"));
        //assert_eq!(ctx.offset_to_position(1), Some(SourcePosition::new(1, 0)));
        //assert_eq!(ctx.offset_to_position(2), Some(SourcePosition::new(1, 1)));

        for offset in 0..=src.len() {
            println!(
                "offset {} in {:?} ({:?}) has position {:?}",
                offset,
                src,
                src.get(offset..offset + 1),
                ctx.offset_to_position(offset as u32)
            );
        }

        let offset = ctx.position_to_offset(SourcePosition::new(2, 0)).unwrap();
        assert_eq!(offset, 4);
        assert_eq!(&src[offset as usize..], "a");
    }

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
