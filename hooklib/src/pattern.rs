use petgraph::{graph, visit::{EdgeRef, NodeCount}};
use std::collections::{linked_list::Cursor, LinkedList};

type Id = graph::NodeIndex;

#[derive(Debug)]
enum Node {
    Stitch {
        ty: &'static str,
    },
    ChainSpace {
        surrounding_nodes: Vec<Id>,
    },
}

impl Node {
    fn chain() -> Self {
        Self::Stitch {
            ty: "ch",
        }
    }

    fn dc() -> Self {
        Self::Stitch {
            ty: "dc",
        }
    }

    fn decrease() -> Self {
        Self::Stitch {
            ty: "dec",
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum EdgeType {
    Previous,
    Insert,
    Slip,
}

#[derive(Default)]
struct Pattern {
    graph: graph::DiGraph<Node, EdgeType>,
    start: Option<graph::NodeIndex>,
    prev: Option<graph::NodeIndex>,
    insert: Option<graph::NodeIndex>,
}

impl Pattern {
    pub fn graph(&self) -> &graph::DiGraph<Node, EdgeType> {
        &self.graph
    }

    pub fn new_row(&mut self) {
        if self.graph.node_count() == 0 {
            self.start = Some(self.graph.add_node(Node::chain()));
            self.prev = self.start;
        } else {
            self.insert = self.prev;
            self.chain();
            self.skip();
        }
    }

    pub fn skip(&mut self) {
        self.insert = self.graph.edges(self.insert.unwrap())
            .find(|e| *e.weight() == EdgeType::Previous)
            .map(|e| e.target());
    }

    pub fn chain(&mut self) {
        let new_node = self.graph.add_node(Node::chain());
        self.graph.add_edge(new_node, self.prev.unwrap(), EdgeType::Previous);
        self.prev = Some(new_node);
    }

    pub fn dc(&mut self) {
        let new_node = self.graph.add_node(Node::dc());
        self.graph.add_edge(new_node, self.prev.unwrap(), EdgeType::Previous);
        self.graph.add_edge(new_node, self.insert.unwrap(), EdgeType::Insert);
        self.skip();

        self.prev = Some(new_node);
    }

    pub fn dc_noskip(&mut self) {
        let new_node = self.graph.add_node(Node::dc());
        self.graph.add_edge(new_node, self.prev.unwrap(), EdgeType::Previous);
        self.graph.add_edge(new_node, self.insert.unwrap(), EdgeType::Insert);

        self.prev = Some(new_node);
    }

    pub fn dec(&mut self) {
        let new_node = self.graph.add_node(Node::decrease());
        self.graph.add_edge(new_node, self.prev.unwrap(), EdgeType::Previous);
        self.graph.add_edge(new_node, self.insert.unwrap(), EdgeType::Insert);
        self.skip();
        self.graph.add_edge(new_node, self.insert.unwrap(), EdgeType::Insert);
        self.skip();

        self.prev = Some(new_node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::dot::{Dot, Config};

    #[test]
    fn test_create_pattern() {
        let mut pattern = Pattern::default();

        pattern.new_row();
        for i in 0..=7 {
            pattern.chain();
        }
        for j in 0..=6 {
            pattern.new_row();
            for i in j..=5 {
                pattern.dc();
            }
            pattern.dec();
        }

//         pattern.new_row();
//         for i in 0..=8 {
//             pattern.dc_noskip();
//             pattern.chain();
//             pattern.dc();
//             pattern.skip();
//         }
// 
//         pattern.new_row();
//         for i in 0..=8 {
//             pattern.dc_noskip();
//             pattern.chain();
//             pattern.dc();
//             pattern.skip();
//             pattern.skip();
//         }

        println!("{:?}", Dot::with_config(pattern.graph(), &[Config::EdgeNoLabel, Config::NodeNoLabel]));
    }

//     #[test]
//     fn test_create_pattern() {
//         let mut row1 = LinkedList::<Node>::new();
//         // base chain
//         for i in 0..=15 {
//             row1.push_back(Node::chain());
//         }
//         row1.push_back(Node::chain());
// 
//         // dc row
//         let mut row2 = LinkedList::<Node>::new();
//         row1.iter().rev().skip(1).for_each(|ch| {
//             row2.push_back(Node::dc(ch.id()));
//         });
//         row2.push_back(Node::chain());
// 
//         // decrease row
//         let mut row3 = LinkedList::<Node>::new();
//         {
//             let mut iter = row2.iter().rev().skip(1);
//             while let Some(s1) = iter.next() {
//                 if let Some(s2) = iter.next() {
//                     row3.push_back(Node::decrease(s1.id(), s2.id()));
//                 } else {
//                     row3.push_back(Node::dc(s1.id()));
//                 }
//             }
//         }
//         row3.push_back(Node::chain());
// 
//         // decrease row
//         let mut row4 = LinkedList::<Node>::new();
//         {
//             let mut iter = row3.iter().rev().skip(1);
//             while let Some(s1) = iter.next() {
//                 if let Some(s2) = iter.next() {
//                     row4.push_back(Node::decrease(s1.id(), s2.id()));
//                 } else {
//                     row4.push_back(Node::dc(s1.id()));
//                 }
//             }
//         }
//         row4.push_back(Node::chain());
// 
//         let pattern = row1.into_iter()
//             .chain(row2)
//             .chain(row3)
//             .chain(row4)
//             .collect::<Vec<Node>>();
// 
//         pattern.iter()
//             .for_each(|s| match s {
//                 Node::Stitch { id, ty, inserts } => println!("{:?}: {:?} into {:?}", id, ty, inserts),
//                 _ => ()
//             })
//     }
}
