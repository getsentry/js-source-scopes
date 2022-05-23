use std::ops::Range;

use rslint_parser::{ast, AstNode, SyntaxKind, SyntaxNode, SyntaxNodeExt, TextRange};

use crate::scope_name::{NameComponent, ScopeName};

pub fn parse_with_rslint(src: &str) -> Vec<(Range<u32>, Option<ScopeName>)> {
    let parse =
        //rslint_parser::parse_with_syntax(src, 0, rslint_parser::FileKind::TypeScript.into());
        rslint_parser::parse_text(src, 0);

    let syntax = parse.syntax();
    // dbg!(&syntax);

    let mut ranges = vec![];

    for node in syntax.descendants() {
        if let Some(fn_decl) = node.try_to::<ast::FnDecl>() {
            ranges.push(node_range_and_name(&node, fn_decl.name()))
        } else if let Some(fn_expr) = node.try_to::<ast::FnExpr>() {
            ranges.push(node_range_and_name(&node, fn_expr.name()))
        } else if let Some(class_decl) = node.try_to::<ast::ClassDecl>() {
            // NOTE: instead of going for the `constructor`, we will cover the
            // whole class body, as class property definitions are executed as
            // part of the constructor.

            ranges.push(node_range_and_name(&node, class_decl.name()));
        } else if let Some(class_expr) = node.try_to::<ast::ClassExpr>() {
            // Same here, see NOTE above.

            ranges.push(node_range_and_name(&node, class_expr.name()));
        } else if node.is::<ast::ArrowExpr>() || node.is::<ast::Method>() {
            ranges.push(node_range_and_name(&node, None));
        }
    }

    ranges
}

fn node_range_and_name(
    node: &SyntaxNode,
    name: Option<ast::Name>,
) -> (Range<u32>, Option<ScopeName>) {
    let mut name = if let Some(name) = name {
        let tokens = name
            .syntax()
            .descendants_with_tokens()
            .filter_map(|el| el.into_token());

        let mut name = ScopeName::new();

        for t in tokens {
            name.components.push_back(NameComponent::token(t));
        }

        Some(name)
    } else {
        find_name_from_ctx(node).map(|n| {
            let mut name = ScopeName::new();
            name.components.push_back(NameComponent::compat(n));
            name
        })
    };

    if node.is::<ast::ClassDecl>() || node.is::<ast::ClassExpr>() {
        if let Some(name) = &mut name {
            name.components.push_front(NameComponent::interp("new "));
        }
    }

    dbg!((convert_text_range(node.text_range()), name))
}

fn convert_text_range(range: TextRange) -> Range<u32> {
    range.start().into()..range.end().into()
}

/// Joins a reverse list of Identifiers using `.`.
fn join_names(names: &[impl std::borrow::Borrow<str>]) -> Option<String> {
    // `Iterator::intersperse` is nightly only :-(
    // .intersperse('.').collect()
    let mut iter = names.iter().rev();
    let mut s = match iter.next() {
        Some(s) => String::from(s.borrow()),
        None => return None,
    };

    for part in iter {
        s.push('.');
        s.push_str(part.borrow());
    }

    Some(s)
}

/// Tries to infer a name for `node` by walking up the chain of ancestors.
fn find_name_from_ctx(node: &SyntaxNode) -> Option<String> {
    // TODO: maybe reuse an allocation here?
    let mut names = vec![];

    if let Some(method_name) = node.try_to::<ast::Method>().and_then(|method| {
        // `ast::Method` has no convenient getter for `PrivateName` :-(
        method
            .name()
            .map(|n| n.text())
            .or_else(|| node.child_with_ast::<ast::PrivateName>().map(|n| n.text()))
    }) {
        names.push(method_name);
    }

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
        if let Some(prop) = parent.try_to::<ast::LiteralProp>() {
            let name = prop.key().map(|n| n.text());
            if let Some(name) = name {
                names.push(name);
            }
        } else if let Some(class_decl) = parent.try_to::<ast::ClassDecl>() {
            let name = class_decl.name().map(|n| n.text());
            if let Some(name) = name {
                names.push(name);
                return join_names(&names);
            }
        } else if let Some(assign_expr) = parent.try_to::<ast::AssignExpr>() {
            if let Some(ast::PatternOrExpr::Expr(expr)) = assign_expr.lhs() {
                names.push(text_of_node(expr.syntax()));
                return join_names(&names);
            }
        } else if let Some(decl) = parent.try_to::<ast::Declarator>() {
            if let Some(ast::Pattern::SinglePattern(sp)) = decl.pattern() {
                let name = sp.name().map(|n| n.text());
                if let Some(name) = name {
                    names.push(name);
                    return join_names(&names);
                }
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
