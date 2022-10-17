# Changelog

## 0.2.0

### Features

- Handle getters, setters, non-identifier property names, and object literals in scope names. ([#13](https://github.com/getsentry/js-source-scopes/pull/13))

### Fixes

- `extract_scope_names` now returns an error if parsing fails, instead of an empty vector of scopes. ([#13](https://github.com/getsentry/js-source-scopes/pull/13))

### Internal

- Switch JS parser from rslint to swc, which is actively maintained. ([#13](https://github.com/getsentry/js-source-scopes/pull/13))

## 0.1.0

Inception

