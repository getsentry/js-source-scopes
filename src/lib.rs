pub use rslint::parse_with_rslint;
//pub use swc::parse_with_swc;

mod rslint {
    use rslint_parser::{ast, AstNode, SyntaxKind, SyntaxNode, SyntaxNodeExt};

    pub fn parse_with_rslint(src: &str) {
        let parse =
            //rslint_parser::parse_with_syntax(src, 0, rslint_parser::FileKind::TypeScript.into());
            rslint_parser::parse_text(src, 0);

        let syntax = parse.syntax();
        dbg!(&syntax);

        for node in syntax.descendants() {
            if let Some(fn_decl) = node.try_to::<ast::FnDecl>() {
                let name = fn_decl
                    .name()
                    .map(|n| n.text())
                    .or_else(|| find_fn_name_from_ctx(&node));
                println!("{:?}: {:?}", name, node.text_range());
            } else if let Some(fn_expr) = node.try_to::<ast::FnExpr>() {
                let name = fn_expr
                    .name()
                    .map(|n| n.text())
                    .or_else(|| find_fn_name_from_ctx(&node));
                println!("{:?}: {:?}", name, node.text_range());
            } else if node.is::<ast::ArrowExpr>() {
                let name = find_fn_name_from_ctx(&node);
                println!("{:?}: {:?}", name, node.text_range());
            }

            // TODO: method, constructor

            /*match node.kind() {
                SyntaxKind::METHOD => todo!(),
                SyntaxKind::CONSTRUCTOR => todo!(),
                _ => todo!(),
            }*/
        }
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
        let src = std::fs::read_to_string("tests/fixtures/simple.js").unwrap();

        parse_with_rslint(&src);
    }
}
