use js_source_scopes::{extract_scope_names, ScopeIndex, SourceContext, SourcePosition};

#[test]
fn resolves_fn_names() {
    let src = std::fs::read_to_string("tests/fixtures/trace/sync.mjs").unwrap();

    let scopes = extract_scope_names(&src);
    dbg!(&scopes);
    let index = ScopeIndex::new(scopes).unwrap();

    let ctx = SourceContext::new(&src).unwrap();

    // NOTE: the browsers use 1-based line/column numbers, while the crates uses
    // 0-based numbers everywhere
    dbg!(index.lookup(ctx.position_to_offset(SourcePosition::new(83, 10)).unwrap()));
    dbg!(index.lookup(ctx.position_to_offset(SourcePosition::new(80, 8)).unwrap()));
    dbg!(index.lookup(ctx.position_to_offset(SourcePosition::new(75, 6)).unwrap()));

    // chrome/firefox disagree on this source stack trace location:
    dbg!(index.lookup(ctx.position_to_offset(SourcePosition::new(26, 19)).unwrap()));
    dbg!(index.lookup(ctx.position_to_offset(SourcePosition::new(26, 2)).unwrap()));

    dbg!(index.lookup(ctx.position_to_offset(SourcePosition::new(11, 2)).unwrap()));
    dbg!(index.lookup(ctx.position_to_offset(SourcePosition::new(7, 2)).unwrap()));
    dbg!(index.lookup(ctx.position_to_offset(SourcePosition::new(3, 2)).unwrap()));
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
