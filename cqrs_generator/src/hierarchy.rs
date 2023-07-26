use crate::{
    contracts::{Export, Statement},
    name::{FullName, split_fullname},
    stmt_builder::StmtBuilder,
};

pub enum Hierarchy {
    Statement(Statement),
    Namespace {
        level: usize,
        name: String,
        children: Vec<Self>,
    },
}

impl Hierarchy {
    pub fn new() -> Self {
        Self::Namespace {
            level: 0,
            name: String::new(),
            children: vec![],
        }
    }

    pub fn append(&mut self, stmt: Statement, full_name: FullName) {
        let Self::Namespace {
            level, children, ..
        } = self
        else {
            panic!("you can append only to the Namespace");
        };

        if full_name.namespaces.len() < *level {
            panic!("should not happen");
        } else if full_name.namespaces.len() == *level {
            children.push(Self::Statement(stmt))
        } else {
            let next_name = &full_name.namespaces[*level];
            let existing = children.iter_mut().find(|h| match h {
                Self::Statement(_) => false,
                Self::Namespace { name, .. } => name == next_name,
            });
            if let Some(next) = existing {
                next.append(stmt, full_name);
            } else {
                let mut next = Self::Namespace {
                    level: *level + 1,
                    name: next_name.clone(),
                    children: vec![],
                };
                next.append(stmt, full_name);
                children.push(next);
            }
        }
    }

    pub fn write_to(self, builder: &mut StmtBuilder) {
        match self {
            Hierarchy::Statement(stmt) => builder.append_statemet(&stmt),
            Hierarchy::Namespace {
                level, children, ..
            } if level == 0 => {
                for c in children.into_iter() {
                    c.write_to(builder);
                }
            }
            Hierarchy::Namespace { name, children, .. } => {
                builder.descend(&name);
                for c in children.into_iter() {
                    c.write_to(builder);
                }
                builder.go_up();
            }
        }
    }
}

impl From<Export> for Hierarchy {
    fn from(value: Export) -> Self {
        let mut hierarchy = Hierarchy::new();
        for stmt in value.statements.into_iter() {
            let full_name = split_fullname(&stmt.name);
            hierarchy.append(stmt, full_name);
        }
        hierarchy
    }
}

impl Into<StmtBuilder> for Hierarchy {
    fn into(self) -> StmtBuilder {
        let mut builder = StmtBuilder::new();
        self.write_to(&mut builder);
        builder
    }
}
