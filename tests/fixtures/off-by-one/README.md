This SourceMap has an "off by one" problem.

Minified positions for tokens use the preceding tokens end rather than
accounting for whitespace and having an exact start.

This looks a little bit like so:

`var| a|=...|function| b()`
... but should rather be (with exact matches):
`var |a|=...|function |b()`
