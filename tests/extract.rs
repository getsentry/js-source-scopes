use js_source_scopes::{extract_scope_names, Scopes};

fn scope_strs(scopes: Scopes) -> Vec<Option<String>> {
    scopes
        .into_iter()
        .map(|s| s.1.map(|n| n.to_string()))
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
        Some("new a.B".into()),
        Some("a.B.foo".into()),
        Some("get a.B.bar".into()),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_anon_obj_literal() {
    let src = r#"
         ({
            named_prop: function named_fun() {},
            anon_prop: function () {},
            arrow_prop: () => {},
            method_prop() {},
        });
        "#;
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        Some("<object>.named_fun".into()),
        Some("<object>.anon_prop".into()),
        Some("<object>.arrow_prop".into()),
        Some("<object>.method_prop".into()),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_empty_function() {
    let src = r#"
        (function () {
          return () => {};
        })()
        "#;
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [None, None];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_nested_iife_objects() {
    // NOTE: This mimicks what react-dom does to transpile JSX children into render tree.
    let src = r#"
        (function () {})({
          children: (function () {})({
            children: (function () {})({
              onSubmitError () {
                throw new Error('wat')
              }
            })
          })
        })
        "#;
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        None,
        Some("<object>.children".into()),
        Some("<object>.children.children".into()),
        Some("<object>.children.children.onSubmitError".into()),
    ];
    assert_eq!(scopes, expected);
}

#[test]
fn extract_computed_properties() {
    let src = r#"         
        Klass.prototype[42] = () => {}
        Klass.prototype["method"] = () => {}
        Klass.prototype[method] = () => {}
        Klass.prototype[1 + 1] = () => {};
        "#;
    let scopes = extract_scope_names(src).unwrap();
    let scopes = scope_strs(scopes);

    let expected = [
        Some("Klass.prototype[42]".into()),
        Some("Klass.prototype[\"method\"]".into()),
        Some("Klass.prototype[method]".into()),
        Some("Klass.prototype[<computed>]".into()),
    ];
    assert_eq!(scopes, expected);
}
