use std::{
    cell::RefCell,
    fmt::Display,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use elsa::FrozenVec;
use itertools::Itertools;

/// A newtype around a shared string to be used for identifier types.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Identifier(Arc<str>);

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<T> for Identifier
where
    T: Into<Arc<str>>,
{
    fn from(value: T) -> Self {
        Identifier(value.into())
    }
}

/// A parametric script, used for visual scripting
#[derive(Default)]
pub struct ParametricPattern {
    root: Option<OperationRef>,
    nodes: FrozenVec<Box<RefCell<Operation>>>,
}

impl ParametricPattern {
    /// Get a list of all identifiers defined in the script
    pub fn defined_identifiers(&self) -> Vec<Identifier> {
        let mut ids = vec![];
        self.walk(&mut |op| {
            if let Operation::Define(identifier, _) = op {
                ids.push(identifier.clone())
            }
        });

        ids.into_iter().unique().collect::<Vec<_>>()
    }

    /// Build the script with a given instruction as the root.
    pub fn build(&mut self, root: OperationRef) {
        self.root.replace(root);
    }

    /// Get the current root instruction of the script.
    pub fn root(&self) -> Option<OperationRef> {
        self.root
    }

    /// Get a reference to an instruction from its ID.
    pub fn get(&self, node: OperationRef) -> Option<impl Deref<Target = Operation> + use<'_>> {
        Some(self.nodes.get(node.0)?.borrow())
    }

    /// Get a mutable reference to an instruction from its ID.
    pub fn get_mut(
        &self,
        node: OperationRef,
    ) -> Option<impl DerefMut<Target = Operation> + use<'_>> {
        Some(self.nodes.get(node.0)?.borrow_mut())
    }

    /// Add a node to the tree.
    fn add_node(&self, operation: Operation) -> OperationRef {
        let new_ref = OperationRef(self.nodes.len());
        self.nodes.push(Box::new(RefCell::new(operation)));
        new_ref
    }

    /// Remove a node from the tree.
    pub fn remove_node(&self, operation: OperationRef) {
        self.walk_mut(&mut |op| {
            if let Operation::Seq(ref mut v) = op {
                if let Some(to_remove) = v.iter().position(|op| operation == *op) {
                    v.remove(to_remove);
                }
            }
        });
    }

    /// Create instruction: a variable definition.
    pub fn define(&self, name: impl Into<Identifier>, op: OperationRef) -> OperationRef {
        self.add_node(Operation::Define(name.into(), op))
    }
    /// Create instruction: a literal value.
    pub fn literal(&self, value: u32) -> OperationRef {
        self.add_node(Operation::Literal(value))
    }
    /// Create instruction: a variable reference.
    pub fn variable(&self, name: impl Into<Identifier>) -> OperationRef {
        self.add_node(Operation::Variable(name.into()))
    }
    /// Create instruction: a function call.
    pub fn call(&self, name: impl Into<Identifier>) -> OperationRef {
        self.add_node(Operation::Call(name.into()))
    }
    /// Create instruction: a group of instructions in sequence.
    pub fn seq(&self, ops: impl IntoIterator<Item = OperationRef>) -> OperationRef {
        self.add_node(Operation::Seq(ops.into_iter().collect()))
    }
    /// Create instruction: a repetition statement.
    pub fn repeat(&self, n: OperationRef, op: OperationRef) -> OperationRef {
        self.add_node(Operation::Repeat(n, op))
    }

    /// Transforms an operation and its children into a textual script.
    fn op_to_script(&self, op: OperationRef) -> String {
        match &*self.nodes[op.0].borrow() {
            Operation::Define(name, op) => format!("let {name} = {}", self.op_to_script(*op)),
            Operation::Literal(value) => format!("{value}"),
            Operation::Variable(name) => format!("{name}"),
            Operation::Call(name) => format!("{name}()"),
            Operation::Seq(v) => format!(
                "{{\n{}\n}}",
                v.iter().map(|op| self.op_to_script(*op)).join(";\n")
            ),
            Operation::Repeat(n, op) => {
                format!("rep {} {}", self.op_to_script(*n), self.op_to_script(*op))
            }
        }
    }

    /// Transforms the whole tree into a textual script.
    pub fn to_script(&self) -> String {
        self.op_to_script(self.root.expect("No root node"))
    }

    /// Walk an operation and its children, performing the function `f` at each iteration.
    fn op_walk(&self, op_id: OperationRef, f: &mut dyn FnMut(&Operation)) {
        let op = &*self.nodes[op_id.0].borrow();
        f(op);
        match op {
            Operation::Define(_, operation) => self.op_walk(*operation, f),
            Operation::Seq(ops) => {
                for op in ops.iter() {
                    self.op_walk(*op, f);
                }
            }
            Operation::Repeat(n, op) => {
                self.op_walk(*n, f);
                self.op_walk(*op, f);
            }
            _ => {}
        }
    }

    /// Walk the whole tree, performing the function `f` at each iteration.
    pub fn walk(&self, f: &mut dyn FnMut(&Operation)) {
        self.op_walk(self.root.expect("No root node"), f);
    }

    /// Walk an operation and its children mutably, performing the function `f` at each iteration.
    fn op_walk_mut(&self, op_id: OperationRef, f: &mut dyn FnMut(&mut Operation)) {
        let op = &mut *self.nodes[op_id.0].borrow_mut();
        f(op);
        match op {
            Operation::Define(_, operation) => self.op_walk_mut(*operation, f),
            Operation::Seq(ops) => {
                for op in ops.iter() {
                    self.op_walk_mut(*op, f);
                }
            }
            Operation::Repeat(n, op) => {
                self.op_walk_mut(*n, f);
                self.op_walk_mut(*op, f);
            }
            _ => {}
        }
    }

    /// Walk the whole tree mutably, performing the function `f` at each iteration.
    pub fn walk_mut(&self, f: &mut dyn FnMut(&mut Operation)) {
        self.op_walk_mut(self.root.expect("No root node"), f);
    }
}

/// A newtype wrapping the index of an operation in the flattened tree.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct OperationRef(usize);

/// An operation in the visual script.
pub enum Operation {
    /// A variable definition
    Define(Identifier, OperationRef),
    /// A literal value
    Literal(u32),
    /// A literal reference
    Variable(Identifier),
    /// A function call
    Call(Identifier),
    /// A group of sequential instructions
    Seq(Vec<OperationRef>),
    /// A repetition statement
    Repeat(OperationRef, OperationRef),
}

/// Returns an example parametric pattern representing a flat sheet of crochet.
pub fn example_flat() -> ParametricPattern {
    let mut p: ParametricPattern = ParametricPattern::default();

    let root = p.seq([
        p.define("stitches", p.literal(15)),
        p.define("x", p.literal(5)),
        p.define("y", p.literal(6)),
        p.repeat(p.variable("stitches"), p.seq([p.call("chain")])),
        p.repeat(
            p.variable("stitches"),
            p.seq([
                p.call("turn"),
                p.repeat(p.variable("stitches"), p.seq([p.call("dc")])),
            ]),
        ),
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
