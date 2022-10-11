
use js_source_scopes::{extract_scope_names, Scopes};

fn scope_strs(scopes: Scopes) -> Vec<Option<String>> {
    scopes
        .into_iter()
        .map(|s| s.1.map(|n| n.to_string()).filter(|s| !s.is_empty()))
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

    let expected = [Some("fn_decl".into()), Some("fn_expr".into())];
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

    let expected = [Some("new class_decl".into()), Some("new class_expr".into())];
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
        Some("anon_fn".into()),
        Some("new anon_class".into()),
        Some("arrow".into()),
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
        Some("assigned_fn".into()),
        Some("new deep.assigned.klass".into()),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_obj_literal() {
    let src = r#"
        const obj_literal = {
            named_prop: function named_fun() {},
            anon_prop: function () {},
            arrow_prop: () => {},
            method_prop() {},
        };
        "#;
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        Some("obj_literal.named_fun".into()),
        Some("obj_literal.anon_prop".into()),
        Some("obj_literal.arrow_prop".into()),
        Some("obj_literal.method_prop".into()),
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
        Some("new class_decl".into()),
        Some("class_decl.static_method".into()),
        Some("class_decl.class_method".into()),
        Some("class_decl.#private_method".into()),
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
        Some("new A".into()),
        Some("get A.foo".into()),
        Some("set A.foo".into()),
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

    let expected = [Some("get a.foo".into()), Some("set a.foo".into())];
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
        Some("a.<computed>".into()),
        Some("a.<1.7>".into()),
        Some("a.<\"bar\">".into()),
        Some("a.<1n>".into()),
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
        Some("new B".into()),
        Some("B.foo".into()),
        Some("get B.bar".into()),
    ];
    assert_eq!(scopes, expected);
    ];
    assert_eq!(scopes, expected);
}
