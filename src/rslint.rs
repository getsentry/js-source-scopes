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
    // dbg!(&syntax);

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
        } else if let Some(class_decl) = node.try_to::<ast::ClassDecl>() {
            // NOTE: instead of going for the `constructor`, we will cover the
            // whole class body, as class property definitions are executed as
            // part of the constructor.

            let name = class_decl
                .name()
                .map(|n| n.text())
                .or_else(|| find_fn_name_from_ctx(&node))
                .map(|mut s| {
                    s.insert_str(0, "new ");
                    s
                });

            ranges.push((convert_text_range(node.text_range()), name));
        } else if let Some(class_expr) = node.try_to::<ast::ClassExpr>() {
            // Same here, see NOTE above.

            let name = class_expr
                .name()
                .map(|n| n.text())
                .or_else(|| find_fn_name_from_ctx(&node))
                .map(|mut s| {
                    s.insert_str(0, "new ");
                    s
                });

            ranges.push((convert_text_range(node.text_range()), name));
        }

        // TODO: method, constructor

        /*match node.kind() {
            SyntaxKind::METHOD => todo!(),
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
