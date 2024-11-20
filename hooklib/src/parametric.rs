use std::{cell::RefCell, fmt::Display, ops::{Deref, DerefMut}, sync::Arc};

use elsa::FrozenVec;
use itertools::Itertools;

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
pub struct ParametricPattern {
    root: Option<OperationRef>,
    nodes: FrozenVec<Box<RefCell<Operation>>>,
}

impl ParametricPattern {
    pub fn defined_identifiers(&self) -> Vec<Identifier> {
        let mut ids = vec![];
        self.walk(&mut |op| match op {
            Operation::Define(identifier, _) => ids.push(identifier.clone()),
            _ => ()
        });

        ids.into_iter()
            .unique()
            .collect::<Vec<_>>()
    }

    pub fn build(&mut self, root: OperationRef) {
        self.root.replace(root);
    }

    pub fn root(&self) -> Option<OperationRef> {
        self.root
    }

    pub fn get(&self, node: OperationRef) -> Option<impl Deref<Target = Operation> + use<'_>> {
        Some(self.nodes.get(node.0)?.borrow())
    }

    pub fn get_mut(&self, node: OperationRef) -> Option<impl DerefMut<Target = Operation> + use<'_>> {
        Some(self.nodes.get(node.0)?.borrow_mut())
    }

    fn add_node(&self, operation: Operation) -> OperationRef {
        let new_ref = OperationRef(self.nodes.len());
        self.nodes.push(Box::new(RefCell::new(operation)));
        new_ref
    }

    pub fn define   (&self, name: impl Into<Identifier>, op: OperationRef) -> OperationRef
        { self.add_node(Operation::Define(name.into(), op)) }
    pub fn literal  (&self, value: u32) -> OperationRef
        { self.add_node(Operation::Literal(value)) }
    pub fn variable (&self, name: impl Into<Identifier>) -> OperationRef
        { self.add_node(Operation::Variable(name.into())) }
    pub fn call     (&self, name: impl Into<Identifier>) -> OperationRef
        { self.add_node(Operation::Call(name.into())) }
    pub fn seq      (&self, ops: impl IntoIterator<Item = OperationRef>) -> OperationRef
        { self.add_node(Operation::Seq(ops.into_iter().collect())) }
    pub fn repeat   (&self, n: OperationRef, op: OperationRef) -> OperationRef
        { self.add_node(Operation::Repeat(n, op)) }

    fn op_to_script(&self, op: OperationRef) -> String {
        match &*self.nodes[op.0].borrow() {
            Operation::Define(name, op) => format!("let {name} = {}", self.op_to_script(*op)),
            Operation::Literal(value) => format!("{value}"),
            Operation::Variable(name) => format!("{name}"),
            Operation::Call(name) => format!("{name}()"),
            Operation::Seq(v) => format!("{{ {} }}", v.iter().map(|op| self.op_to_script(*op)).join(";")),
            Operation::Repeat(n, op) => format!("{} # || {}", self.op_to_script(*n), self.op_to_script(*op)),
        }
    }

    pub fn to_script(&self) -> String {
        self.op_to_script(self.root.expect("No root node"))
    }

    fn op_walk(&self, op_id: OperationRef, f: &mut dyn FnMut(&Operation)) {
        let op = &*self.nodes[op_id.0].borrow();
        f(op);
        match op {
            Operation::Define(_, operation) => self.op_walk(*operation, f),
            Operation::Seq(ops) => {
                for op in ops.iter() {
                    self.op_walk(*op, f);
                }
            },
            Operation::Repeat(n, op) => {
                self.op_walk(*n, f);
                self.op_walk(*op, f);
            },
            _ => {}
        }
    }

    pub fn walk(&self, f: &mut dyn FnMut(&Operation)) {
        self.op_walk(self.root.expect("No root node"), f);
    }
}

#[derive(Clone, Copy, Hash)]
pub struct OperationRef(usize);

pub enum Operation {
    Define(Identifier, OperationRef),
    Literal(u32),
    Variable(Identifier),
    Call(Identifier),
    Seq(Vec<OperationRef>),
    Repeat(OperationRef, OperationRef),
}

pub fn example_flat<'a>() -> ParametricPattern {
    let mut p: ParametricPattern = ParametricPattern::default();

    let root = p.seq([
            p.define("stitches", p.literal(15)),
            p.define("x", p.literal(5)),
            p.define("y", p.literal(6)),
            p.repeat(p.variable("stitches"), p.call("chain")),
            p.repeat(p.variable("stitches"), p.seq([
                p.call("turn"),
                p.repeat(p.variable("stitches"), p.call("dc"))
            ])),
        ]);
    
    p.build(root);

    p
}

#[cfg(test)]
mod tests {
    use crate::parametric::*;

    #[test]
    fn test_example_flat() {
        println!("{}", example_flat().to_script())
    }
}