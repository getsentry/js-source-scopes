use std::collections::VecDeque;
use std::fmt::Display;
use std::ops::Range;

use rslint_parser::SyntaxToken;

#[derive(Debug)]
pub struct ScopeName {
    pub(crate) components: VecDeque<NameComponent>,
}

impl ScopeName {
    pub(crate) fn new() -> Self {
        Self {
            components: Default::default(),
        }
    }

    pub fn components(&self) -> impl Iterator<Item = &NameComponent> + '_ {
        self.components.iter()
    }
}

impl Display for ScopeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.components() {
            f.write_str(c.text())?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct NameComponent {
    pub(crate) inner: NameComponentInner,
}

impl NameComponent {
    pub fn text(&self) -> &str {
        match &self.inner {
            NameComponentInner::Interpolation(s) => s,
            NameComponentInner::SourceToken(t) => t.text().as_str(),
            NameComponentInner::Compat(s) => s,
        }
    }

    pub fn range(&self) -> Option<Range<u32>> {
        todo!()
    }

    pub(crate) fn interp(s: &'static str) -> Self {
        Self {
            inner: NameComponentInner::Interpolation(s),
        }
    }
    pub(crate) fn token(token: SyntaxToken) -> Self {
        Self {
            inner: NameComponentInner::SourceToken(token),
        }
    }
    pub(crate) fn compat(s: String) -> Self {
        Self {
            inner: NameComponentInner::Compat(s),
        }
    }
}

#[derive(Debug)]
pub(crate) enum NameComponentInner {
    Interpolation(&'static str),
    SourceToken(SyntaxToken),
    Compat(String),
}
