use std::ops::Range;

use js_source_scopes::{
    extract_scope_names, NameResolver, ScopeIndex, ScopeLookupResult, ScopeName, SourceContext,
    SourcePosition,
};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap()
}

fn resolve_original_scopes(
    minified: &str,
    map: &str,
    scopes: Vec<(Range<u32>, Option<ScopeName>)>,
) -> Vec<(Range<u32>, Option<String>, Option<String>)> {
    let ctx = SourceContext::new(&minified).unwrap();

    let sm = sourcemap::decode_slice(map.as_bytes()).unwrap();

    let resolver = NameResolver::new(&ctx, &sm);

    let resolved_scopes = scopes.into_iter().map(|(range, name)| {
        let minified_name = name.as_ref().map(|n| n.to_string());
        let original_name = name.map(|n| resolver.resolve_name(&n));

        (range, minified_name, original_name)
    });
    resolved_scopes.collect()
}

#[test]
fn resolves_scopes_simple() {
    let minified = std::fs::read_to_string("tests/fixtures/simple/minified.js").unwrap();
    let map = std::fs::read_to_string("tests/fixtures/simple/minified.js.map").unwrap();

    let scopes = extract_scope_names(&minified).unwrap();

    let resolved_scopes = resolve_original_scopes(&minified, &map, scopes);

    assert_eq!(
        resolved_scopes,
        [(0..14, Some("t".into()), Some("abcd".into()))]
    );
}

#[test]
fn resolves_scope_names() {
    let src = std::fs::read_to_string("tests/fixtures/trace/sync.mjs").unwrap();

    let scopes = extract_scope_names(&src).unwrap();
    // dbg!(&scopes);

    let scopes: Vec<_> = scopes
        .into_iter()
        .map(|s| (s.0, s.1.map(|n| n.to_string()).filter(|s| !s.is_empty())))
        .collect();
    let index = ScopeIndex::new(scopes).unwrap();

    let ctx = SourceContext::new(&src).unwrap();

    use ScopeLookupResult::*;
    let lookup = |l: u32, c: u32| {
        // NOTE: the browsers use 1-based line/column numbers, while the crates uses
        // 0-based numbers everywhere
        let offset = ctx
            .position_to_offset(SourcePosition::new(l - 1, c - 1))
            .unwrap();
        index.lookup(offset)
    };

    // objectLiteralAnon@http://127.0.0.1:8080/sync.mjs:84:11
    // at Object.objectLiteralAnon (http://127.0.0.1:8080/sync.mjs:84:11)
    assert_eq!(lookup(84, 11), NamedScope("obj.objectLiteralAnon"));

    // objectLiteralMethod@http://127.0.0.1:8080/sync.mjs:81:9
    // at Object.objectLiteralMethod (http://127.0.0.1:8080/sync.mjs:81:9)
    assert_eq!(lookup(81, 9), NamedScope("obj.objectLiteralMethod"));

    // localReassign@http://127.0.0.1:8080/sync.mjs:76:7
    // at localReassign (http://127.0.0.1:8080/sync.mjs:76:7)
    assert_eq!(lookup(76, 7), NamedScope("localReassign"));

    // Klass.prototype.prototypeMethod@http://127.0.0.1:8080/sync.mjs:71:28
    // at Klass.prototypeMethod (http://127.0.0.1:8080/sync.mjs:71:28)
    assert_eq!(
        lookup(71, 28),
        NamedScope("Klass.prototype.prototypeMethod")
    );

    // #privateMethod@http://127.0.0.1:8080/sync.mjs:40:10
    // at Klass.#privateMethod (http://127.0.0.1:8080/sync.mjs:40:10)
    assert_eq!(lookup(40, 10), NamedScope("BaseKlass.#privateMethod"));

    // classCallbackArrow@http://127.0.0.1:8080/sync.mjs:36:24
    // at Klass.classCallbackArrow (http://127.0.0.1:8080/sync.mjs:36:24)
    assert_eq!(lookup(36, 24), NamedScope("BaseKlass.classCallbackArrow"));

    // classCallbackBound/<@http://127.0.0.1:8080/sync.mjs:65:34
    // at http://127.0.0.1:8080/sync.mjs:65:34
    // TODO: should we infer a better name here?
    assert_eq!(lookup(65, 34), AnonymousScope);

    // classCallbackBound@http://127.0.0.1:8080/sync.mjs:65:22
    // at Klass.classCallbackBound (http://127.0.0.1:8080/sync.mjs:65:5)
    assert_eq!(lookup(65, 22), NamedScope("Klass.classCallbackBound"));
    assert_eq!(lookup(65, 5), NamedScope("Klass.classCallbackBound"));

    // classCallbackSelf@http://127.0.0.1:8080/sync.mjs:61:22
    // at Klass.classCallbackSelf (http://127.0.0.1:8080/sync.mjs:61:5)
    assert_eq!(lookup(61, 22), NamedScope("Klass.classCallbackSelf"));
    assert_eq!(lookup(61, 5), NamedScope("Klass.classCallbackSelf"));

    // classMethod/<@http://127.0.0.1:8080/sync.mjs:56:12
    // at http://127.0.0.1:8080/sync.mjs:56:12
    // TODO: should we infer a better name here?
    assert_eq!(lookup(56, 12), AnonymousScope);

    // classMethod@http://127.0.0.1:8080/sync.mjs:55:22
    // at Klass.classMethod (http://127.0.0.1:8080/sync.mjs:55:5)
    assert_eq!(lookup(55, 22), NamedScope("Klass.classMethod"));
    assert_eq!(lookup(55, 5), NamedScope("Klass.classMethod"));

    // BaseKlass@http://127.0.0.1:8080/sync.mjs:32:10
    // at new BaseKlass (http://127.0.0.1:8080/sync.mjs:32:10)
    assert_eq!(lookup(32, 10), NamedScope("new BaseKlass"));

    // Klass@http://127.0.0.1:8080/sync.mjs:50:5
    // at new Klass (http://127.0.0.1:8080/sync.mjs:50:5)
    assert_eq!(lookup(50, 5), NamedScope("new Klass"));

    // staticMethod@http://127.0.0.1:8080/sync.mjs:46:5
    // at Function.staticMethod (http://127.0.0.1:8080/sync.mjs:46:5)
    assert_eq!(lookup(46, 5), NamedScope("Klass.staticMethod"));

    // arrowFn/namedDeclaredCallback/namedImmediateCallback/</<@http://127.0.0.1:8080/sync.mjs:22:17
    // at http://127.0.0.1:8080/sync.mjs:22:17
    // TODO: should we infer a better name here?
    assert_eq!(lookup(22, 17), AnonymousScope);

    // arrowFn/namedDeclaredCallback/namedImmediateCallback/<@http://127.0.0.1:8080/sync.mjs:21:26
    // at http://127.0.0.1:8080/sync.mjs:21:9
    // TODO: should we infer a better name here?
    assert_eq!(lookup(21, 26), AnonymousScope);
    assert_eq!(lookup(21, 9), AnonymousScope);

    // namedImmediateCallback@http://127.0.0.1:8080/sync.mjs:19:24
    // at namedImmediateCallback (http://127.0.0.1:8080/sync.mjs:19:7)
    assert_eq!(lookup(19, 24), NamedScope("namedImmediateCallback"));
    assert_eq!(lookup(19, 7), NamedScope("namedImmediateCallback"));

    // namedDeclaredCallback@http://127.0.0.1:8080/sync.mjs:17:22
    // at namedDeclaredCallback (http://127.0.0.1:8080/sync.mjs:17:5)
    assert_eq!(lookup(17, 22), NamedScope("namedDeclaredCallback"));
    assert_eq!(lookup(17, 5), NamedScope("namedDeclaredCallback"));

    // arrowFn@http://127.0.0.1:8080/sync.mjs:27:20
    // at arrowFn (http://127.0.0.1:8080/sync.mjs:27:3)
    assert_eq!(lookup(27, 20), NamedScope("arrowFn"));
    assert_eq!(lookup(27, 3), NamedScope("arrowFn"));

    // anonFn@http://127.0.0.1:8080/sync.mjs:12:3
    // at anonFn (http://127.0.0.1:8080/sync.mjs:12:3)
    assert_eq!(lookup(12, 3), NamedScope("anonFn"));

    // namedFnExpr@http://127.0.0.1:8080/sync.mjs:8:3
    // at namedFnExpr (http://127.0.0.1:8080/sync.mjs:8:3)
    assert_eq!(lookup(8, 3), NamedScope("namedFnExpr"));

    // namedFn@http://127.0.0.1:8080/sync.mjs:4:3
    // at namedFn (http://127.0.0.1:8080/sync.mjs:4:3)
    assert_eq!(lookup(4, 3), NamedScope("namedFn"));
}

#[test]
fn resolves_token_from_names() {
    let minified = std::fs::read_to_string("tests/fixtures/preact.module.js").unwrap();
    let map = std::fs::read_to_string("tests/fixtures/preact.module.js.map").unwrap();

    let scopes = extract_scope_names(&minified).unwrap();

    let resolved_scopes = resolve_original_scopes(&minified, &map, scopes);

    for (range, minified, original) in resolved_scopes {
        println!("{range:?}");
        println!("  minified: {minified:?}");
        println!("  original: {original:?}");
    }
}

#[test]
fn parses_large_vendors() {
    let minified = std::fs::read_to_string("tests/fixtures/vendors/vendors.js").unwrap();
    let map = std::fs::read_to_string("tests/fixtures/vendors/vendors.js.map").unwrap();

    let scopes = extract_scope_names(&minified).unwrap();

    let resolved_scopes = resolve_original_scopes(&minified, &map, scopes);

    for (range, minified, original) in resolved_scopes {
        println!("{range:?}");
        println!("  minified: {minified:?}");
        println!("  original: {original:?}");
    }
}

#[test]
fn should_not_resolve_mismatch() {
    let minified = fixture("prototype-chain/minified.js");
    let map = fixture("prototype-chain/minified.js.map");

    let scopes = extract_scope_names(&minified).unwrap();

    // The sourcemap was manually edited to remove the `prototype` token/name.
    // When resolving names, we should make sure we have exact matches, otherwise
    // we would use the preceding token and end up with `Foo.Foo.bar`.
    let resolved_scopes = resolve_original_scopes(&minified, &map, scopes);

    assert_eq!(resolved_scopes[1].2, Some("Foo.prototype.bar".into()));
}

#[test]
fn should_resolve_exactish() {
    let minified = fixture("off-by-one/test.min.js");
    let map = fixture("off-by-one/test.map");

    let scopes = extract_scope_names(&minified).unwrap();

    let resolved_scopes = resolve_original_scopes(&minified, &map, scopes);

    assert_eq!(resolved_scopes[2].2, Some("onFailure".into()));
    assert_eq!(resolved_scopes[3].2, Some("invoke".into()));
    assert_eq!(resolved_scopes[4].2, Some("test".into()));
}
