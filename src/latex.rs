use std::{iter::Peekable, sync::LazyLock};

use strum::{EnumIter, EnumString, IntoEnumIterator, IntoStaticStr, VariantNames};
use trie_rs::map::{Trie, TrieBuilder};

use crate::parser::tokenizer::{Token, TokenizerError};

pub static LATEX_TRIE: LazyLock<Trie<char, LatexExpr>> = LazyLock::new(|| {
    let mut builder = TrieBuilder::new();
    for v in LatexExpr::iter() {
        let s: &'static str = v.clone().into();
        builder.insert(s.chars(), v)
    }
    builder.build()
});

pub static EXPECTED_LATEX: LazyLock<String> = LazyLock::new(|| {
    LatexExpr::VARIANTS
        .iter()
        .map(|s| format!(", {}", s))
        .collect()
});

pub fn tokenize_latex(
    chars: &mut Peekable<impl Iterator<Item = char>>,
) -> Result<Token, TokenizerError> {
    if let Some((_, latex)) = LATEX_TRIE
        .common_prefix_search::<String, _, _>(chars)
        .next()
    {
        Ok(Token::Latex(latex.clone()))
    } else {
        Err(TokenizerError::InvalidLatex)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Eq, Hash, PartialEq, VariantNames, EnumIter, EnumString, IntoStaticStr)]
pub enum LatexExpr {
    sin,
    log,
    ln,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Symbol {
    Char(char),
    Latex(LatexExpr),
}

impl From<char> for Symbol {
    fn from(value: char) -> Self {
        Self::Char(value)
    }
}

impl From<LatexExpr> for Symbol {
    fn from(value: LatexExpr) -> Self {
        Self::Latex(value)
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SubscriptSymbol {
    pub c: Symbol,
    pub subscript: Option<String>,
}

impl From<Symbol> for SubscriptSymbol {
    fn from(value: Symbol) -> Self {
        Self {
            c: value,
            subscript: None,
        }
    }
}
impl From<char> for SubscriptSymbol {
    fn from(value: char) -> Self {
        Self {
            c: value.into(),
            subscript: None,
        }
    }
}
