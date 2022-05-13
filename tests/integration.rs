use js_source_scopes::{
    extract_scope_names, ScopeIndex, ScopeLookupResult, SourceContext, SourcePosition,
};

#[test]
fn resolves_fn_names() {
    let src = std::fs::read_to_string("tests/fixtures/trace/sync.mjs").unwrap();

    let scopes = extract_scope_names(&src);
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
    // TODO:
    // assert_eq!(lookup(84, 11), NamedScope(""));

    // objectLiteralMethod@http://127.0.0.1:8080/sync.mjs:81:9
    // at Object.objectLiteralMethod (http://127.0.0.1:8080/sync.mjs:81:9)
    // TODO:
    // assert_eq!(lookup(81, 9), NamedScope(""));

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
    // TODO:
    // assert_eq!(lookup(40, 10), NamedScope("BaseKlass.#privateMethod"));

    // classCallbackArrow@http://127.0.0.1:8080/sync.mjs:36:24
    // at Klass.classCallbackArrow (http://127.0.0.1:8080/sync.mjs:36:24)
    // TODO:
    // assert_eq!(lookup(36, 24), NamedScope("Klass.classCallbackArrow"));

    // classCallbackBound/<@http://127.0.0.1:8080/sync.mjs:65:34
    // at http://127.0.0.1:8080/sync.mjs:65:34
    // TODO: should we infer a better name here?
    assert_eq!(lookup(65, 34), AnonymousScope);
    assert_eq!(lookup(65, 34), AnonymousScope);

    // classCallbackBound@http://127.0.0.1:8080/sync.mjs:65:22
    // at Klass.classCallbackBound (http://127.0.0.1:8080/sync.mjs:65:5)
    // TODO:
    // assert_eq!(lookup(65, 22), NamedScope("Klass.classCallbackSelf"));
    // assert_eq!(lookup(65, 5), NamedScope("Klass.classCallbackSelf"));

    // classCallbackSelf@http://127.0.0.1:8080/sync.mjs:61:22
    // at Klass.classCallbackSelf (http://127.0.0.1:8080/sync.mjs:61:5)
    // TODO:
    // assert_eq!(lookup(61, 22), NamedScope("Klass.classCallbackSelf"));
    // assert_eq!(lookup(61, 5), NamedScope("Klass.classCallbackSelf"));

    // classMethod/<@http://127.0.0.1:8080/sync.mjs:56:12
    // at http://127.0.0.1:8080/sync.mjs:56:12
    // TODO: should we infer a better name here?
    assert_eq!(lookup(56, 12), AnonymousScope);

    // classMethod@http://127.0.0.1:8080/sync.mjs:55:22
    // at Klass.classMethod (http://127.0.0.1:8080/sync.mjs:55:5)
    // TODO:
    // assert_eq!(lookup(55, 22), NamedScope("Klass.classMethod"));
    // assert_eq!(lookup(55, 5), NamedScope("Klass.classMethod"));

    // BaseKlass@http://127.0.0.1:8080/sync.mjs:32:10
    // at new BaseKlass (http://127.0.0.1:8080/sync.mjs:32:10)
    assert_eq!(lookup(32, 10), NamedScope("new BaseKlass"));

    // Klass@http://127.0.0.1:8080/sync.mjs:50:5
    // at new Klass (http://127.0.0.1:8080/sync.mjs:50:5)
    assert_eq!(lookup(50, 5), NamedScope("new Klass"));

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

/*
firefox:
# sync stack trace
objectLiteralAnon@http://127.0.0.1:8080/sync.mjs:84:11
objectLiteralMethod@http://127.0.0.1:8080/sync.mjs:81:9
localReassign@http://127.0.0.1:8080/sync.mjs:76:7
Klass.prototype.prototypeMethod@http://127.0.0.1:8080/sync.mjs:71:28
#privateMethod@http://127.0.0.1:8080/sync.mjs:40:10
classCallbackArrow@http://127.0.0.1:8080/sync.mjs:36:24
classCallbackBound/<@http://127.0.0.1:8080/sync.mjs:65:34
callsSyncCallback@http://127.0.0.1:8080/shared.mjs:2:3
classCallbackBound@http://127.0.0.1:8080/sync.mjs:65:22
callsSyncCallback@http://127.0.0.1:8080/shared.mjs:2:3
classCallbackSelf@http://127.0.0.1:8080/sync.mjs:61:22
classMethod/<@http://127.0.0.1:8080/sync.mjs:56:12
callsSyncCallback@http://127.0.0.1:8080/shared.mjs:2:3
classMethod@http://127.0.0.1:8080/sync.mjs:55:22
BaseKlass@http://127.0.0.1:8080/sync.mjs:32:10
Klass@http://127.0.0.1:8080/sync.mjs:50:5
staticMethod@http://127.0.0.1:8080/sync.mjs:46:5
arrowFn/namedDeclaredCallback/namedImmediateCallback/</<@http://127.0.0.1:8080/sync.mjs:22:17
callsSyncCallback@http://127.0.0.1:8080/shared.mjs:2:3
arrowFn/namedDeclaredCallback/namedImmediateCallback/<@http://127.0.0.1:8080/sync.mjs:21:26
callsSyncCallback@http://127.0.0.1:8080/shared.mjs:2:3
namedImmediateCallback@http://127.0.0.1:8080/sync.mjs:19:24
callsSyncCallback@http://127.0.0.1:8080/shared.mjs:2:3
namedDeclaredCallback@http://127.0.0.1:8080/sync.mjs:17:22
callsSyncCallback@http://127.0.0.1:8080/shared.mjs:2:3
arrowFn@http://127.0.0.1:8080/sync.mjs:27:20
anonFn@http://127.0.0.1:8080/sync.mjs:12:3
namedFnExpr@http://127.0.0.1:8080/sync.mjs:8:3
namedFn@http://127.0.0.1:8080/sync.mjs:4:3
@http://127.0.0.1:8080/trace.mjs:10:3


# async stack trace
asyncObjectLiteralAnon@http://127.0.0.1:8080/async.mjs:32:11
asyncObjectLiteralMethod@http://127.0.0.1:8080/async.mjs:29:20
AsyncKlass.prototype.asyncProtoMethod@http://127.0.0.1:8080/async.mjs:24:18
#privateAsyncMethod@http://127.0.0.1:8080/async.mjs:19:16
asyncClassMethod@http://127.0.0.1:8080/async.mjs:16:35
asyncStaticMethod@http://127.0.0.1:8080/async.mjs:13:13
asyncArrowFn@http://127.0.0.1:8080/async.mjs:7:20
asyncNamedFn@http://127.0.0.1:8080/async.mjs:4:9
@http://127.0.0.1:8080/trace.mjs:20:9
*/

/*
chrome:
# sync stack trace
Error
    at Object.objectLiteralAnon (http://127.0.0.1:8080/sync.mjs:84:11)
    at Object.objectLiteralMethod (http://127.0.0.1:8080/sync.mjs:81:9)
    at localReassign (http://127.0.0.1:8080/sync.mjs:76:7)
    at Klass.prototypeMethod (http://127.0.0.1:8080/sync.mjs:71:28)
    at Klass.#privateMethod (http://127.0.0.1:8080/sync.mjs:40:10)
    at Klass.classCallbackArrow (http://127.0.0.1:8080/sync.mjs:36:24)
    at http://127.0.0.1:8080/sync.mjs:65:34
    at callsSyncCallback (http://127.0.0.1:8080/shared.mjs:2:3)
    at Klass.classCallbackBound (http://127.0.0.1:8080/sync.mjs:65:5)
    at callsSyncCallback (http://127.0.0.1:8080/shared.mjs:2:3)
    at Klass.classCallbackSelf (http://127.0.0.1:8080/sync.mjs:61:5)
    at http://127.0.0.1:8080/sync.mjs:56:12
    at callsSyncCallback (http://127.0.0.1:8080/shared.mjs:2:3)
    at Klass.classMethod (http://127.0.0.1:8080/sync.mjs:55:5)
    at new BaseKlass (http://127.0.0.1:8080/sync.mjs:32:10)
    at new Klass (http://127.0.0.1:8080/sync.mjs:50:5)
    at Function.staticMethod (http://127.0.0.1:8080/sync.mjs:46:5)
    at http://127.0.0.1:8080/sync.mjs:22:17
    at callsSyncCallback (http://127.0.0.1:8080/shared.mjs:2:3)
    at http://127.0.0.1:8080/sync.mjs:21:9
    at callsSyncCallback (http://127.0.0.1:8080/shared.mjs:2:3)
    at namedImmediateCallback (http://127.0.0.1:8080/sync.mjs:19:7)
    at callsSyncCallback (http://127.0.0.1:8080/shared.mjs:2:3)
    at namedDeclaredCallback (http://127.0.0.1:8080/sync.mjs:17:5)
    at callsSyncCallback (http://127.0.0.1:8080/shared.mjs:2:3)
    at arrowFn (http://127.0.0.1:8080/sync.mjs:27:3)
    at anonFn (http://127.0.0.1:8080/sync.mjs:12:3)
    at namedFnExpr (http://127.0.0.1:8080/sync.mjs:8:3)
    at namedFn (http://127.0.0.1:8080/sync.mjs:4:3)
    at http://127.0.0.1:8080/trace.mjs:10:3

# async stack trace
Error
    at Object.asyncObjectLiteralAnon (http://127.0.0.1:8080/async.mjs:32:11)
    at Object.asyncObjectLiteralMethod (http://127.0.0.1:8080/async.mjs:29:20)
    at AsyncKlass.asyncProtoMethod (http://127.0.0.1:8080/async.mjs:24:18)
    at AsyncKlass.#privateAsyncMethod (http://127.0.0.1:8080/async.mjs:19:16)
    at AsyncKlass.asyncClassMethod (http://127.0.0.1:8080/async.mjs:16:35)
    at Function.asyncStaticMethod (http://127.0.0.1:8080/async.mjs:13:13)
    at asyncArrowFn (http://127.0.0.1:8080/async.mjs:7:20)
    at asyncNamedFn (http://127.0.0.1:8080/async.mjs:4:9)
    at http://127.0.0.1:8080/trace.mjs:20:9
*/
