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
pub struct SourcePosition {
    /// Line in the source file, 0-based.
    line: u32,
    /// Column in the source file, 0-based.
    ///
    /// The column is given in UTF-16 code points.
    column: u32,
}

/// A Source Context allowing fast access to lines and line/column <-> byte offset remapping.
pub struct SourceContext<T> {
    src: T,
    line_offsets: Vec<u32>,
}

/// An Error that can happen when building a [`SourceContext`].
pub struct SourceContextError(());

impl<T: AsRef<str>> SourceContext<T> {
    /// Construct a new Source Context from the given `src` buffer.
    pub fn new(src: T) -> Result<Self, SourceContextError> {
        let buf = src.as_ref();
        let buf_ptr = buf.as_ptr();
        let mut line_offsets: Vec<u32> = buf
            .lines()
            .skip(1)
            .map(|line| unsafe { line.as_ptr().offset_from(buf_ptr) as usize }.try_into())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| SourceContextError(()))?;
        line_offsets.push(buf.len().try_into().map_err(|_| SourceContextError(()))?);
        Ok(Self { src, line_offsets })
    }

    /// Get the `nth` line of the source, 0-based.
    pub fn get_line(&self, nth: u32) -> Option<&str> {
        let from = if nth == 0 {
            0
        } else {
            self.line_offsets.get((nth - 1) as usize).copied()? as usize
        };
        let to = self.line_offsets.get(nth as usize).copied()? as usize;
        self.src.as_ref().get(from..to)
    }

    /// Converts a byte offset into the source to the corresponding line/column.
    ///
    /// The column is given in UTF-16 code points.
    pub fn offset_to_position(&self, offset: u32) -> Option<SourcePosition> {
        let line_no = match self.line_offsets.binary_search(&offset) {
            Ok(line) => line,
            Err(line) => line,
        }
        .try_into()
        .ok()?;
        let mut byte_offset = self.line_offsets.get(line_no as usize).copied()? as usize;
        let line = self.get_line(line_no)?;

        let mut utf16_offset = 0;
        for c in line.chars() {
            utf16_offset += c.len_utf16();
            byte_offset += c.len_utf8();

            if byte_offset >= offset as usize {
                return Some(SourcePosition {
                    line: line_no,
                    column: utf16_offset.try_into().ok()?,
                });
            }
        }

        None
    }

    /// Converts the given line/column to the corresponding byte offset inside the source.
    pub fn position_to_offset(&self, position: SourcePosition) -> Option<u32> {
        let SourcePosition { line, column } = position;

        let from = if line == 0 {
            0
        } else {
            self.line_offsets.get((line - 1) as usize).copied()? as usize
        };
        let to = self.line_offsets.get(line as usize).copied()? as usize;

        let line = self.src.as_ref().get(from..to)?;

        let mut byte_offset = to;
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
    fn parses() {
        let src = std::fs::read_to_string("tests/fixtures/trace/sync.mjs").unwrap();

        let scopes = extract_scope_names(&src);
        dbg!(scopes);
    }
}
