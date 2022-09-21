# JS Source Scopes

[![Build Status](https://github.com/getsentry/js-source-scopes/workflows/CI/badge.svg)](https://github.com/getsentry/js-source-scopes/actions?workflow=CI)
<a href="https://crates.io/crates/js-source-scopes"><img src="https://img.shields.io/crates/v/js-source-scopes.svg" alt=""></a>
<a href="https://github.com/getsentry/js-source-scopes/blob/master/LICENSE"><img src="https://img.shields.io/crates/l/js-source-scopes.svg" alt=""></a>
[![codecov](https://codecov.io/gh/getsentry/js-source-scopes/branch/master/graph/badge.svg?token=nKJzvC8nog)](https://codecov.io/gh/getsentry/js-source-scopes)

This crate provides functionality for extracting and processing scope information from JavaScript source files,
and resolving that scope via SourceMaps.

# Features

- Extracting scopes from source text using [`extract_scope_names`]
- Fast lookup of scopes by byte offset using [`ScopeIndex`]
- Fast conversion between line/column source positions and byte offsets using [`SourceContext`]
- Resolution of minified scope names to their original names using [`NameResolver`]

## License

JS Source Scopes is licensed under the Apache-2.0 license.
