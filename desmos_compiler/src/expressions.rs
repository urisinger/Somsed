use anyhow::{anyhow, Result};

use std::collections::HashMap;

use parse::{
    ast::ExpressionListEntry,
    ast_parser::parse_expression_list_entry,
    latex_parser::parse_latex,
    latex_tree_flattener::{flatten, Token},
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpressionId(pub u32);

impl From<u32> for ExpressionId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Default)]
pub struct Expressions {
    pub exprs: HashMap<ExpressionId, ExpressionListEntry>,
    pub idents: HashMap<String, ExpressionId>,
}

impl Expressions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remove_expr(&mut self, id: impl Into<ExpressionId>) {
        let k = id.into();
        if let Some(
            ExpressionListEntry::Assignment { name, .. }
            | ExpressionListEntry::FunctionDeclaration { name, .. },
        ) = self.exprs.remove(&k)
        {
            self.idents.remove(&name);
        }
    }

    pub fn insert_expr(&mut self, id: impl Into<ExpressionId>, s: &str) -> Result<()> {
        let k = id.into();
        self.parse_expr(s, k).map(|expr| {
            self.exprs.insert(k, expr);
        })
    }

    pub fn get_expr(&self, name: &str) -> Option<&ExpressionListEntry> {
        if let Some(id) = self.idents.get(name) {
            self.exprs.get(id)
        } else {
            None
        }
    }

    fn parse_expr(&mut self, s: &str, k: ExpressionId) -> Result<ExpressionListEntry> {
        let expr = parse_str_into_expression_list_entry(s)?;
        match &expr {
            ExpressionListEntry::Assignment { name, .. }
            | ExpressionListEntry::FunctionDeclaration { name, .. } => {
                self.idents.insert(name.to_string(), k);
            }
            _ => (),
        };

        Ok(expr)
    }
}

pub fn parse_str_into_expression_list_entry(s: &str) -> Result<ExpressionListEntry> {
    let nodes = parse_latex(s).map_err(|e| anyhow!("{e:?}"))?;
    let expression = parse_expression_list_entry(&nodes).map_err(|e| anyhow!("{e}"))?;
    Ok(expression)
}
