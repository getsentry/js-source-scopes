use std::ops::Range;

use js_source_scopes::{extract_scope_names, Scopes};

fn scope_strs(scopes: Scopes) -> Vec<(Range<u32>, Option<String>)> {
    scopes
        .into_iter()
        .map(|s| (s.0, s.1.map(|n| n.to_string()).filter(|s| !s.is_empty())))
        .collect()
}

#[test]
fn extracts_named_fn() {
    let src = r#"
        function fn_decl() {
            return function fn_expr() {};
        }
        "#;
    let scopes = extract_scope_names(src);
    let scopes = scope_strs(scopes);

    assert_eq!(scopes[0], (9..81, Some("fn_decl".into())));
    assert_eq!(scopes[1], (49..70, Some("fn_expr".into())));
}

#[test]
fn extracts_named_class() {
    let src = r#"
        class class_decl {
            constructor() {
                return class class_expr {};
            }
        }
        "#;
    let scopes = extract_scope_names(src);
    let scopes = scope_strs(scopes);

    assert_eq!(scopes[0], (9..123, Some("new class_decl".into())));
    assert_eq!(scopes[1], (79..98, Some("new class_expr".into())));
}

#[test]
fn infer_from_decl() {
    let src = r#"
        var anon_fn = function () {};
        let anon_class = class {};
        const arrow = () => {};
        "#;
    let scopes = extract_scope_names(src);
    let scopes = scope_strs(scopes);

    assert_eq!(scopes[0], (23..37, Some("anon_fn".into())));
    assert_eq!(scopes[1], (64..72, Some("new anon_class".into())));
    assert_eq!(scopes[2], (96..104, Some("arrow".into())));
}

#[test]
fn infer_from_assign() {
    let src = r#"
        assigned_fn = function () {};
        deep.assigned.klass = class {};
        "#;
    let scopes = extract_scope_names(src);
    let scopes = scope_strs(scopes);

    assert_eq!(scopes[0], (23..37, Some("assigned_fn".into())));
    assert_eq!(scopes[1], (69..77, Some("new deep.assigned.klass".into())));
}
