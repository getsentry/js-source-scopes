# Changelog

## 0.6.0

### Various fixes & improvements

- deps: Update (#28) by @loewenheim
- feat(release): Replace release bot with GH app (#26) by @Jeffreyhung

## 0.5.0

### Various fixes & improvements

- Update dependencies (#25) by @Swatinem

## 0.4.0

### Various fixes & improvements

- update dependencies with incompatible versions (#23) by @demoray

## 0.3.2

### Various fixes & improvements

- Update all `swc` crates to their latest version (#22) by @Swatinem

## 0.3.1

### Various fixes & improvements

- fix: Use saturating_sub in try_map_token to prevent overflow (#21) by @kamilogorek

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
