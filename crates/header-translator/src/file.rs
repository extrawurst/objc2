use std::fmt;

use crate::config::Config;
use crate::stmt::Stmt;
use crate::FILE_PRELUDE;

#[derive(Debug, PartialEq)]
pub struct File {
    imports: Vec<String>,
    stmts: Vec<Stmt>,
}

impl File {
    pub fn new(config: &Config) -> Self {
        Self {
            imports: config.imports.clone(),
            stmts: Vec::new(),
        }
    }

    pub fn declared_types(&self) -> impl Iterator<Item = &str> {
        self.stmts
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::ClassDecl { ty, .. } => Some(&*ty.name),
                Stmt::Methods { .. } => None,
                Stmt::ProtocolDecl { name, .. } => Some(&*name),
                Stmt::ProtocolImpl { .. } => None,
                Stmt::StructDecl { name, .. } => Some(&*name),
                Stmt::EnumDecl { name, .. } => name.as_deref(),
                Stmt::VarDecl { name, .. } => Some(&*name),
                Stmt::FnDecl { name, body, .. } if body.is_none() => Some(&*name),
                // TODO
                Stmt::FnDecl { .. } => None,
                Stmt::AliasDecl { name, .. } => Some(&*name),
            })
            .chain(self.stmts.iter().flat_map(|stmt| {
                if let Stmt::EnumDecl { variants, .. } = stmt {
                    variants.iter().map(|(name, _)| &**name).collect()
                } else {
                    vec![]
                }
            }))
    }

    pub fn add_stmt(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }

    pub fn compare(&self, other: &Self) {
        super::compare_vec(&self.stmts, &other.stmts, |_i, self_stmt, other_stmt| {
            self_stmt.compare(other_stmt);
        });
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{FILE_PRELUDE}")?;

        writeln!(f, "use crate::common::*;")?;
        for import in &self.imports {
            writeln!(f, "use crate::{import}::*;")?;
        }

        writeln!(f, "")?;

        for stmt in &self.stmts {
            writeln!(f, "{stmt}")?;
        }

        Ok(())
    }
}
