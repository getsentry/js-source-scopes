This fixture reproduces a TypeScript source map pattern where the original function
name is attached to the `function` keyword token rather than to the identifier.

The source map has these segments around the function declaration:

  col 7 (within `function` keyword) → original col 9, name = "initServer"
  col 8 (space)                     → original col 9, name = (none)
  [no segment at col 9 where `ab` starts]

When looking up col 9 (the `ab` identifier), the nearest token is at col 8
(no name). The fix checks col 7 (one before), finds it maps to the same
original source position, and uses its name "initServer".

The source map was generated programmatically to mimic TypeScript compiler output.
The real-world case this tests: TypeScript compiling `function initServer()` to
`function Uc1bk()`, where Sentry's symbolicator was unable to resolve the name.
