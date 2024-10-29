use petgraph::{graph, visit::EdgeRef, Direction};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Node {
    Stitch {
        ty: &'static str,
    },
    ChainSpace,
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

    fn ch_sp() -> Self {
        Self::ChainSpace
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EdgeType {
    Previous,
    Insert,
    Slip,
    Neighbour,
}

impl Into<f32> for EdgeType {
    fn into(self) -> f32 {
        match self {
            EdgeType::Previous => 1.0,
            EdgeType::Insert => 0.75,
            EdgeType::Slip => 0.000001,
            EdgeType::Neighbour => 1.0,
        }
    }
}

#[derive(Default, Debug)]
pub struct Pattern {
    graph: graph::DiGraph<Node, EdgeType>,
    start: Option<graph::NodeIndex>,
    prev: Option<graph::NodeIndex>,
    insert: Option<graph::NodeIndex>,
    current_ch_sp: Option<Vec<graph::NodeIndex>>,
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        petgraph::algo::is_isomorphic_matching(self.graph(), other.graph(), PartialEq::eq, PartialEq::eq)
    }
}

impl Pattern {
    pub fn graph(&self) -> &graph::DiGraph<Node, EdgeType> {
        &self.graph
    }

    pub fn prev(&self) -> Option<graph::NodeIndex> {
        self.prev
    }

    pub fn set_insert(&mut self, insert: graph::NodeIndex) {
        self.insert = Some(insert);
    }

    pub fn to_graphviz(&self) -> String {
        use petgraph::dot::{Dot, Config};

        let node_attr_getter = |_g, (id, &ref n)| {
            let options = match n {
                Node::Stitch { ty: "ch" } => "shape = \"ellipse\" scale = 0.5 label = \"\"",
                Node::Stitch { ty: "dc" } => "shape = \"none\" label = \"+\" margin = \"0\" fontsize = 56.0",
                _ => "shape = \"point\" label = \"\""
            };
            let style = if id == self.start.unwrap() { "filled" } else { "" };

            format!("{options} style=\"{style}\"")
        };

        let dot = Dot::with_attr_getters(
            self.graph(),
            &[Config::EdgeNoLabel, Config::NodeNoLabel, Config::GraphContentOnly],
            &|_g, e| match e.weight() {
                EdgeType::Previous => "len = 1.0",
                EdgeType::Insert => r#"len = 1.0 style = "dotted" arrowhead="vee""#,
                EdgeType::Slip => "len = 1.0 style = \"dashed\"",
                EdgeType::Neighbour => "len = 1.0 style = \"invis\"",
            }.into(),
            &node_attr_getter,
        );

        format!("digraph {{\n    normalize = 180\n{:?}}}", dot)
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

    pub fn new_row_noskip(&mut self) {
        if self.graph.node_count() == 0 {
            self.start = Some(self.graph.add_node(Node::chain()));
            self.prev = self.start;
        } else {
            self.insert = self.prev;
            self.chain();
        }
    }

    pub fn skip(&mut self) {
        self.insert = self.graph.edges_directed(self.insert.unwrap(), Direction::Outgoing)
            .find(|e| *e.weight() == EdgeType::Previous)
            .map(|e| e.target());
    }

    pub fn skip_rev(&mut self) {
        self.insert = self.graph.edges_directed(self.insert.unwrap(), Direction::Incoming)
            .find(|e| *e.weight() == EdgeType::Previous)
            .map(|e| e.source());
    }

    pub fn chain(&mut self) {
        let new_node = self.graph.add_node(Node::chain());
        self.graph.add_edge(new_node, self.prev.unwrap(), EdgeType::Previous);
        self.prev = Some(new_node);

        if let Some(ch_sp) = self.current_ch_sp.as_mut() {
            ch_sp.push(new_node);
        }
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
    
    pub fn dec_rev(&mut self) {
        let new_node = self.graph.add_node(Node::decrease());
        self.graph.add_edge(new_node, self.prev.unwrap(), EdgeType::Previous);
        self.graph.add_edge(new_node, self.insert.unwrap(), EdgeType::Insert);
        self.skip_rev();
        self.graph.add_edge(new_node, self.insert.unwrap(), EdgeType::Insert);
        self.skip_rev();

        self.prev = Some(new_node);
    }

    pub fn inc(&mut self) {
        self.dc_noskip();
        self.dc();
    }

    pub fn slip_stitch(&mut self, into: graph::NodeIndex) {
        self.graph.add_edge(self.prev.unwrap(), into, EdgeType::Slip);
    }

    pub fn start_ch_sp(&mut self) {
        if self.current_ch_sp.is_some() {
            panic!("tried to start a chain space while one was already started!");
        }

        self.current_ch_sp = Some(vec![self.prev.unwrap()]);
    }

    pub fn end_ch_sp(&mut self) -> graph::NodeIndex {
        let ch_sp = self.current_ch_sp.take()
            .expect("tried to end a chain space while none was started!");

        let new_node = self.graph.add_node(Node::ch_sp());
        ch_sp.into_iter()
            .for_each(|neighbour| {
                self.graph.add_edge(new_node, neighbour, EdgeType::Neighbour);
            });

        new_node
    }
}

pub fn test_pattern() -> Pattern {
    let mut pattern = Pattern::default();

    pattern.new_row();
    pattern.start_ch_sp();
    let start = pattern.prev().unwrap();
    for _ in 1..=2 {
        pattern.chain();
    }
    pattern.slip_stitch(start);
    let ch_sp = pattern.end_ch_sp();

    pattern.set_insert(ch_sp);
    let start = pattern.prev().unwrap();
    for _ in 1..=5 {
        pattern.dc_noskip();
    }
    pattern.set_insert(start);

    for _ in 1..=6 {
        pattern.dc_noskip();
        pattern.dc_noskip();
        pattern.skip_rev();
    }
    
    for j in 1..20 {
        for _ in 1..=6 {
            for _ in 1..=j {
                pattern.dc_noskip();
                pattern.skip_rev();
            }
            pattern.dc_noskip();
            pattern.dc_noskip();
            pattern.skip_rev();
        }
    }

    pattern
}

pub fn test_pattern_sphere() -> Pattern {
    let mut pattern = Pattern::default();

    pattern.new_row();
    pattern.start_ch_sp();
    let start = pattern.prev().unwrap();
    for _ in 1..=2 {
        pattern.chain();
    }
    pattern.slip_stitch(start);
    let ch_sp = pattern.end_ch_sp();

    pattern.set_insert(ch_sp);
    let start = pattern.prev().unwrap();
    for _ in 1..=5 {
        pattern.dc_noskip();
    }
    pattern.set_insert(start);
    
    for j in 0..=4 {
        for _ in 1..=6 {
            for _ in 1..=j {
                pattern.dc_noskip();
                pattern.skip_rev();
            }
            pattern.dc_noskip();
            pattern.dc_noskip();
            pattern.skip_rev();
        }
    }

    for _ in 1..=7 {
        for _ in 1..=36 {
            pattern.dc_noskip();
            pattern.skip_rev();
        }
    }

    for j in (0..=4).rev() {
        for _ in 1..=6 {
            for _ in 1..=j {
                pattern.dc_noskip();
                pattern.skip_rev();
            }
            pattern.dec_rev();
        }
    }

    pattern.dec_rev();
    pattern.dec_rev();

    pattern
}

pub fn test_pattern_2() -> Pattern {
    let mut pattern = Pattern::default();

    pattern.new_row();
    let start = pattern.prev().unwrap();
    for _ in 1..=5 {
        pattern.chain();
    }
    pattern.slip_stitch(start);

    pattern.new_row_noskip();
    let start = pattern.prev().unwrap();
    pattern.dc();
    for _ in 1..=5 {
        pattern.inc();
    }
    pattern.slip_stitch(start);

    for round in 1..=20 {
        pattern.new_row();
        let start = pattern.prev().unwrap();
        for _ in 1..=5 {
            pattern.inc();
            for _ in 1..=round {
                pattern.dc();
            }
        }
        pattern.inc();
        for _ in 1..round {
            pattern.dc();
        }
        pattern.slip_stitch(start);
    }

    pattern
}

pub fn test_pattern_3() -> Pattern {
    let mut pattern = Pattern::default();

    pattern.new_row();
    for _ in 1..=15 {
        pattern.chain();
    }
    for _ in 1..=15 {
        pattern.new_row();
        for _ in 1..=15 {
            pattern.dc();
        }
    }

    pattern
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const TEST_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_out");

    #[test]
    fn test_create_pattern() {
        let mut pattern = Pattern::default();

        pattern.new_row();
        for _ in 1..=15 {
            pattern.chain();
        }
        for _ in 1..=15 {
            pattern.new_row();
            for _ in 1..=15 {
                pattern.dc();
            }
        }

        let mut file = std::fs::File::create(format!("{TEST_DIR}/test.dot")).unwrap();
        write!(file, "{}", pattern.to_graphviz()).unwrap();
    }

    #[test]
    fn test_spiral_rounds() {
        let mut pattern = Pattern::default();

        pattern.new_row();
        pattern.start_ch_sp();
        let start = pattern.prev().unwrap();
        for _ in 1..=2 {
            pattern.chain();
        }
        pattern.slip_stitch(start);
        let ch_sp = pattern.end_ch_sp();

        pattern.set_insert(ch_sp);
        let start = pattern.prev().unwrap();
        for _ in 1..=5 {
            pattern.dc_noskip();
        }
        pattern.set_insert(start);

        for _ in 1..=6 {
            pattern.dc_noskip();
            pattern.dc_noskip();
            pattern.skip_rev();
        }
        
        for j in 1..20 {
            for _ in 1..=6 {
                for _ in 1..=j {
                    pattern.dc_noskip();
                    pattern.skip_rev();
                }
                pattern.dc_noskip();
                pattern.dc_noskip();
                pattern.skip_rev();
            }
        }
        

        let mut file = std::fs::File::create(format!("{TEST_DIR}/spiral.dot")).unwrap();
        write!(file, "{}", pattern.to_graphviz()).unwrap();
    }

    #[test]
    fn test_rounds() {
        let mut pattern = Pattern::default();

        pattern.new_row();
        let start = pattern.prev().unwrap();
        for _ in 1..=5 {
            pattern.chain();
        }
        pattern.slip_stitch(start);

        pattern.new_row_noskip();
        let start = pattern.prev().unwrap();
        pattern.dc();
        for _ in 1..=5 {
            pattern.inc();
        }
        pattern.slip_stitch(start);

        for round in 1..=20 {
            pattern.new_row();
            let start = pattern.prev().unwrap();
            for _ in 1..=5 {
                pattern.inc();
                for _ in 1..=round {
                    pattern.dc();
                }
            }
            pattern.inc();
            for _ in 1..round {
                pattern.dc();
            }
            pattern.slip_stitch(start);
        }

        let mut file = std::fs::File::create(format!("{TEST_DIR}/rounds.dot")).unwrap();
        write!(file, "{}", pattern.to_graphviz()).unwrap();
    }
}
