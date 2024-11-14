use std::{cell::RefCell, fmt::Display, sync::Arc};

use itertools::Itertools;
use typed_arena::Arena;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Identifier(Arc<str>);

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<T> for Identifier
where T: Into<Arc<str>> {
    fn from(value: T) -> Self {
        Identifier(value.into())
    }
}

#[derive(Default)]
pub struct ParametricPattern<'ar> {
    root: RefCell<Option<OperationRef<'ar>>>,
    nodes: Arena<Operation<'ar>>,
}

impl<'ar> ParametricPattern<'ar> {
    pub fn defined_identifiers(&self) -> Vec<Identifier> {
        let mut ids = vec![];
        self.walk(|op| match op {
            Operation::Define(identifier, _) => ids.push(identifier.clone()),
            _ => ()
        });

        ids.into_iter()
            .unique()
            .collect::<Vec<_>>()
    }

    pub fn build(&'ar self, root: OperationRef<'ar>) {
        self.root.replace(Some(root));
    }

    fn add_node(&'ar self, operation: Operation<'ar>) -> OperationRef<'ar> {
        OperationRef(self.nodes.alloc(operation))
    }

    pub fn define   (&'ar self, name: impl Into<Identifier>, op: OperationRef<'ar>) -> OperationRef<'ar>
        { self.add_node(Operation::Define(name.into(), op)) }
    pub fn literal  (&'ar self, value: u32) -> OperationRef<'ar>
        { self.add_node(Operation::Literal(value)) }
    pub fn variable (&'ar self, name: impl Into<Identifier>) -> OperationRef<'ar>
        { self.add_node(Operation::Variable(name.into())) }
    pub fn call     (&'ar self, name: impl Into<Identifier>) -> OperationRef<'ar>
        { self.add_node(Operation::Call(name.into())) }
    pub fn seq      (&'ar self, ops: impl IntoIterator<Item = OperationRef<'ar>>) -> OperationRef<'ar>
        { self.add_node(Operation::Seq(ops.into_iter().collect())) }
    pub fn repeat   (&'ar self, n: OperationRef<'ar>, op: OperationRef<'ar>) -> OperationRef<'ar>
        { self.add_node(Operation::Repeat(n, op)) }

    fn op_to_script(&self, op: OperationRef) -> String {
        match op.0 {
            Operation::Define(name, op) => format!("let {name} = {}", self.op_to_script(*op)),
            Operation::Literal(value) => format!("{value}"),
            Operation::Variable(name) => format!("{name}"),
            Operation::Call(name) => format!("{name}()"),
            Operation::Seq(v) => format!("{{ {} }}", v.iter().map(|op| self.op_to_script(*op)).join(";")),
            Operation::Repeat(n, op) => format!("{} # || {}", self.op_to_script(*n), self.op_to_script(*op)),
        }
    }

    pub fn to_script(&self) -> String {
        self.op_to_script(self.root.borrow().expect("No root node"))
    }

    fn op_walk<F>(&self, op_id: OperationRef, mut f: F)
    where F: FnMut(&Operation) {
        let op = op_id.0;
        f(op);
        match op {
            Operation::Define(_, operation) => self.op_walk(*operation, &mut f),
            Operation::Seq(ops) => ops.iter().for_each(|op| self.op_walk(*op, &mut f)),
            Operation::Repeat(n, op) => {
                self.op_walk(*n, &mut f);
                self.op_walk(*op, &mut f);
            },
            _ => {}
        }
    }

    pub fn walk<F>(&self, f: F)
    where F: FnMut(&Operation) {
        self.op_walk(self.root.borrow().expect("No root node"), f);
    }
}

#[derive(Clone, Copy)]
pub struct OperationRef<'ar>(&'ar Operation<'ar>);

pub enum Operation<'ar> {
    Define(Identifier, OperationRef<'ar>),
    Literal(u32),
    Variable(Identifier),
    Call(Identifier),
    Seq(Vec<OperationRef<'ar>>),
    Repeat(OperationRef<'ar>, OperationRef<'ar>),
}

pub fn example_flat() -> ParametricPattern<'static> {
    let pattern = ParametricPattern::default();

    let root = pattern.seq([
            pattern.define("stitches", pattern.literal(15)),
            pattern.repeat(pattern.variable("stitches"), pattern.call("chain")),
            pattern.repeat(pattern.variable("stitches"), pattern.seq([
                pattern.call("turn"),
                pattern.repeat(pattern.variable("stitches"), pattern.call("dc"))
            ])),
        ]);
    
    pattern.build(root);

    pattern
}

#[cfg(test)]
mod tests {
    use crate::parametric::*;

    #[test]
    fn test_example_flat() {
        println!("{}", example_flat().to_script())
    }
}