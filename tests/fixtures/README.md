# Minifying code and creating SourceMaps:

For example:

```
./node_modules/.bin/terser -c -m --module tests/fixtures/simple/original.js --source-map includeSources -o tests/fixtures/simple/minified.js
```

# trace

A sync and async stack trace through various constructs, with named and anonymous
functions and arrow functions.

# perf

- https://unpkg.com/typescript@4.6.4/lib/typescript.js
- https://unpkg.com/preact@10.7.2/dist/preact.module.js
