# Changelog

## 0.3.0

### Features

- Handle computed properties in scope names. ([#18](https://github.com/getsentry/js-source-scopes/pull/18))

### Fixes

- Correctly handle scope names for nested object literals. ([#19](https://github.com/getsentry/js-source-scopes/pull/19))

## 0.2.2

### Various fixes & improvements

- fix: Allow an off-by-one in Token matching (#15) by @Swatinem

## 0.2.1

### Various fixes & improvements

- fix: Ensure exact token match in NameResolver (#14) by @Swatinem

## 0.2.0

### Features

- Handle getters, setters, non-identifier property names, and object literals in scope names. ([#13](https://github.com/getsentry/js-source-scopes/pull/13))

### Fixes

- `extract_scope_names` now returns an error if parsing fails, instead of an empty vector of scopes. ([#13](https://github.com/getsentry/js-source-scopes/pull/13))

### Internal

- Switch JS parser from rslint to swc, which is actively maintained. ([#13](https://github.com/getsentry/js-source-scopes/pull/13))

## 0.1.0

Inception
