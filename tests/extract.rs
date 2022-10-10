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
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (9..81, Some("fn_decl".into())),
        (49..70, Some("fn_expr".into())),
    ];
    assert_eq!(scopes, expected);
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
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (9..123, Some("new class_decl".into())),
        (79..98, Some("new class_expr".into())),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn infer_from_decl() {
    let src = r#"
        var anon_fn = function () {};
        let anon_class = class {};
        const arrow = () => {};
        "#;
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (23..37, Some("anon_fn".into())),
        (64..72, Some("new anon_class".into())),
        (96..104, Some("arrow".into())),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn infer_from_assign() {
    let src = r#"
        assigned_fn = function () {};
        deep.assigned.klass = class {};
        "#;
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (23..37, Some("assigned_fn".into())),
        (69..77, Some("new deep.assigned.klass".into())),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_obj_literal() {
    let src = r#"
        const obj_literal = {
            named_prop: function named_prop() {},
            anon_prop: function () {},
            arrow_prop: () => {},
            method_prop() {},
        };
        "#;
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (55..79, Some("named_prop".into())),
        (104..118, Some("obj_literal.anon_prop".into())),
        (144..152, Some("obj_literal.arrow_prop".into())),
        (166..182, Some("obj_literal.method_prop".into())),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_method_names() {
    let src = r#"
        class class_decl {
            static static_method() {}
            class_method() {}
            #private_method() {}
        }
        "#;
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (9..138, Some("new class_decl".into())),
        (40..65, Some("class_decl.static_method".into())),
        (78..95, Some("class_decl.class_method".into())),
        (108..128, Some("class_decl.#private_method".into())),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_class_getter_setter() {
    let src = r#"
      class A {
        get foo() {}
        set foo(x) {}
      }
    "#;

    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (7..67, Some("new A".into())),
        (25..37, Some("get A.foo".into())),
        (46..59, Some("set A.foo".into())),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_object_getter_setter() {
    let src = r#"
      a = {
        get foo() {},
        set foo(x) {}
      }  
    "#;

    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (21..33, Some("get a.foo".into())),
        (43..56, Some("set a.foo".into())),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_object_weird_properties() {
    let src = r#"
      a = {
        ["foo" + 123]() {},
        1.7() {},
        "bar"() {},
        1n() {}
      }
    "#;

    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (21..39, Some("a.<computed property name>".into())),
        (49..57, Some("a.<1.7>".into())),
        (67..77, Some("a.<\"bar\">".into())),
        (87..94, Some("a.<1n>".into())),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_named_class_expr() {
    let src = r#"
      a = class B {
         foo() {}
         get bar() {}
      }  
    "#;

    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        (11..68, Some("new B".into())),
        (30..38, Some("B.foo".into())),
        (48..60, Some("get B.bar".into())),
    ];
    assert_eq!(scopes, expected);
}
