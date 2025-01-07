use std::error::Error;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};

use glam::Vec3;
use itertools::Itertools;
use petgraph::{
    graph::{self, NodeIndex},
    visit::EdgeRef,
    Direction,
};

#[derive(Debug)]
pub enum PatternError {
    EndOfRow,
    NoInsert,
    NoRows,
    SewInvalidLengths,
    NestedChainSpace,
    NoChainSpace,
}

impl Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfRow => write!(f, "Reached end of row when trying to skip. Make sure to use `new_row()` before each row."),
            Self::NoInsert => write!(f, "No insert set. Make sure to use `into()` for magic rings or chain spaces."),
            Self::NoRows => write!(f, "No rows created. Make sure to use `new_row()` before each row."),
            Self::SewInvalidLengths => write!(f, "Rows to sew are not the same length."),
            Self::NestedChainSpace => write!(f, "Tried to start a chain space when one was already started."),
            Self::NoChainSpace => write!(f, "Tried to end a chain space when none was started."),
        }
    }
}

impl Error for PatternError {}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Node {
    Stitch { ty: &'static str, turn: bool, color: Vec3 },
    ChainSpace,
    MagicRing,
}

impl Node {
    fn chain(color: Vec3) -> Self {
        Self::Stitch {
            ty: "ch",
            turn: false,
            color,
        }
    }

    fn turn(color: Vec3) -> Self {
        Self::Stitch {
            ty: "ch",
            turn: true,
            color,
        }
    }

    fn dc(color: Vec3) -> Self {
        Self::Stitch {
            ty: "dc",
            turn: false,
            color,
        }
    }

    fn decrease(color: Vec3) -> Self {
        Self::Stitch {
            ty: "dec",
            turn: false,
            color,
        }
    }

    fn ch_sp() -> Self {
        Self::ChainSpace
    }

    pub fn is_turn(&self) -> bool {
        match self {
            Node::Stitch { turn, .. } => *turn,
            _ => false,
        }
    }

    pub fn stitch_type(&self) -> &'static str {
        match self {
            Node::Stitch { ty, .. } => ty,
            Node::ChainSpace => "ch_sp",
            Node::MagicRing => "magic_ring",
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Node::chain(Vec3::ONE)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EdgeType {
    Previous,
    Insert,
    Slip,
    Neighbour,
    Sew,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
enum SkipDirection {
    #[default]
    Forward,
    Reverse,
}

/// Gauge is the ratio of rows in a given length to stitches in a given length.
pub const GAUGE: f32 = 15.0 / 18.5;

/// Epsilon is just a really small distance, used for stitches that should be really close together (e.g. slips and sews)
pub const EPSILON: f32 = 0.001;

impl From<EdgeType> for f32 {
    fn from(edge_type: EdgeType) -> Self {
        match edge_type {
            EdgeType::Previous => 1.0,
            EdgeType::Insert => GAUGE,
            EdgeType::Slip => EPSILON,
            EdgeType::Neighbour => 1.0,
            EdgeType::Sew => EPSILON,
        }
    }
}

#[derive(Default, Debug)]
pub struct Pattern {
    graph: RwLock<graph::DiGraph<Node, EdgeType>>,
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        petgraph::algo::is_isomorphic_matching(
            &*self.graph.read().unwrap(),
            &*other.graph.read().unwrap(),
            PartialEq::eq,
            PartialEq::eq,
        )
    }
}

impl Pattern {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            graph: Default::default(),
        })
    }

    pub fn into_inner(self: Arc<Self>) -> Self {
        Arc::try_unwrap(self).unwrap_or_else(|s| Pattern {
            graph: s.graph.read().unwrap().clone().into(),
        })
    }

    pub fn add_part(self: &Arc<Self>) -> Part {
        Part::new_from_parent(self.clone())
    }

    pub fn sew(
        &self,
        row_1: Vec<graph::NodeIndex>,
        row_2: Vec<graph::NodeIndex>,
    ) -> Result<(), PatternError> {
        if row_1.len() == row_2.len() {
            let mut graph_mut = self.graph.write().unwrap();
            row_1.into_iter().zip(row_2).for_each(|(node_1, node_2)| {
                graph_mut.add_edge(node_1, node_2, EdgeType::Sew);
            });
            Ok(())
        } else {
            Err(PatternError::SewInvalidLengths)
        }
    }

    pub fn graph(&self) -> impl Deref<Target = graph::DiGraph<Node, EdgeType>> + use<'_> {
        self.graph.read().unwrap()
    }

    pub fn triangulated_graph(&self) -> graph::DiGraph<(), f32> {
        let new_graph = self.graph.read().unwrap().clone();
        let diag_length = (1.0 + (GAUGE * GAUGE)).sqrt();

        let diagonals = new_graph
            .edge_references()
            .filter_map(|p| {
                if *p.weight() == EdgeType::Insert
                    && !new_graph.node_weight(p.target()).unwrap().is_turn()
                {
                    if let Some(endpoint_1) = new_graph
                        .edges_directed(p.source(), Direction::Incoming)
                        .find(|e| *e.weight() == EdgeType::Previous)
                        .map(|e| e.source())
                    {
                        if let Some(endpoint_2) = new_graph
                            .edges_directed(endpoint_1, Direction::Outgoing)
                            .find(|e| *e.weight() == EdgeType::Insert)
                            .map(|e| e.target())
                        {
                            Some(vec![
                                (endpoint_1, p.target(), diag_length),
                                (endpoint_2, p.source(), diag_length),
                            ])
                        } else {
                            Some(vec![(endpoint_1, p.target(), diag_length)])
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        let mut new_graph = new_graph.map(
            |_ix, _node| (),
            |ix, edge| {
                let (start, end) = new_graph.edge_endpoints(ix).unwrap();
                let start = *new_graph.node_weight(start).unwrap();
                let end = *new_graph.node_weight(end).unwrap();
                let edge_length_type = match edge {
                    EdgeType::Previous => {
                        if start.stitch_type() == "ch" && end.stitch_type() == "dc" {
                            EdgeType::Insert
                        } else {
                            EdgeType::Previous
                        }
                    }
                    other => *other,
                };
                edge_length_type.into()
            },
        );

        new_graph.extend_with_edges(diagonals);

        new_graph
    }

    pub fn to_graphviz(&self) -> String {
        use petgraph::dot::{Config, Dot};

        let node_attr_getter = |_g, (_, n): (graph::NodeIndex, &Node)| {
            let options = match n {
                Node::Stitch { ty: "ch", .. } => "shape = \"ellipse\" scale = 0.5 label = \"\"",
                Node::Stitch { ty: "dc", .. } => {
                    "shape = \"none\" label = \"+\" margin = \"0\" fontsize = 56.0"
                }
                _ => "shape = \"point\" label = \"\"",
            };

            options.to_owned()
        };

        let graph = self.graph();
        let dot = Dot::with_attr_getters(
            &*graph,
            &[
                Config::EdgeNoLabel,
                Config::NodeNoLabel,
                Config::GraphContentOnly,
            ],
            &|_g, e| {
                let len: f32 = (*e.weight()).into();
                match e.weight() {
                    EdgeType::Previous => format!("len = {len}"),
                    EdgeType::Insert => format!(r#"len = {len} style = "dotted" arrowhead="vee""#),
                    EdgeType::Slip => format!("len = {len} style = \"dashed\""),
                    EdgeType::Neighbour => format!("len = {len} style = \"invis\""),
                    EdgeType::Sew => format!("len = {len} style = \"dashed\""),
                }
            },
            &node_attr_getter,
        );

        format!("digraph {{\n    normalize = 180\n{:?}}}", dot)
    }
}

#[derive(Debug)]
pub struct Part {
    parent: Arc<Pattern>,

    start: graph::NodeIndex,
    prev: graph::NodeIndex,
    insert: Option<graph::NodeIndex>,
    current_ch_sp: Option<Vec<graph::NodeIndex>>,
    rows: Vec<Vec<graph::NodeIndex>>,
    direction: SkipDirection,
    ignore_for_row: bool,
    current_color: Vec3,
}

impl Part {
    fn graph(&self) -> impl Deref<Target = graph::DiGraph<Node, EdgeType>> + use<'_> {
        self.parent.graph()
    }

    fn graph_mut(&self) -> impl DerefMut<Target = graph::DiGraph<Node, EdgeType>> + use<'_> {
        self.parent.graph.write().unwrap()
    }

    pub fn prev(&self) -> graph::NodeIndex {
        self.prev
    }

    pub fn start(&self) -> graph::NodeIndex {
        self.start
    }

    pub fn current_row(&self) -> Result<&Vec<graph::NodeIndex>, PatternError> {
        self.rows
            .last()
            .ok_or(PatternError::NoRows)
    }

    pub fn current_row_mut(&mut self) -> Result<&mut Vec<graph::NodeIndex>, PatternError> {
        self.rows
            .last_mut()
            .ok_or(PatternError::NoRows)
    }

    pub fn set_insert(&mut self, insert: graph::NodeIndex) {
        self.insert = Some(insert);
    }

    pub fn insert(&self) -> Option<graph::NodeIndex> {
        self.insert
    }

    pub fn new_from_parent(parent: Arc<Pattern>) -> Self {
        let start = parent.graph.write().unwrap().add_node(Node::chain(Vec3::ONE));
        let prev = start;
        let rows = vec![vec![start]];

        Self {
            parent,
            start,
            prev,
            insert: None,
            current_ch_sp: None,
            rows,
            direction: Default::default(),
            ignore_for_row: false,
            current_color: Vec3::ONE,
        }
    }

    pub fn magic_ring(&mut self) {
        self.graph_mut().remove_node(self.start);
        let new_start = self.graph_mut().add_node(Node::MagicRing);
        self.start = new_start;
        self.prev = new_start;
    }

    pub fn new_row(&mut self) -> Result<(), PatternError> {
        self.rows.push(vec![]);
        self.set_insert(
            *self.rows[self.rows.len().checked_sub(2).ok_or(PatternError::NoRows)?]
                .first()
                .ok_or(PatternError::EndOfRow)?,
        );

        Ok(())
    }

    pub fn turn(&mut self) -> Result<(), PatternError> {
        self.turn_noskip()?;
        self.skip()?;

        Ok(())
    }

    pub fn turn_noskip(&mut self) -> Result<(), PatternError> {
        self.new_row()?;
        self.insert = Some(self.prev);
        self.direction = SkipDirection::Reverse;
        let new_node = self.graph_mut().add_node(Node::turn(self.current_color));
        self.graph_mut()
            .add_edge(new_node, self.prev, EdgeType::Previous);
        self.current_row_mut()?
            .push(new_node);
        self.prev = new_node;

        Ok(())
    }

    pub fn skip(&mut self) -> Result<(), PatternError> {
        let insert_row = &self.rows[self.rows.len().checked_sub(2).ok_or(PatternError::NoRows)?];
        let insert = self.insert.ok_or(PatternError::NoInsert)?;
        let curr_insert_idx = insert_row
            .iter()
            .find_position(|s| **s == insert)
            .ok_or(PatternError::EndOfRow)?
            .0;
        if self.direction == SkipDirection::Forward {
            self.insert = insert_row.get(curr_insert_idx + 1).copied();
        } else {
            self.insert = curr_insert_idx
                .checked_sub(1)
                .and_then(|i| insert_row.get(i).copied())
        }

        Ok(())
    }

    pub fn chain(&mut self) -> Result<NodeIndex, PatternError> {
        let new_node = self.graph_mut().add_node(Node::chain(self.current_color));
        self.graph_mut()
            .add_edge(new_node, self.prev, EdgeType::Previous);
        self.prev = new_node;

        if let Some(ch_sp) = self.current_ch_sp.as_mut() {
            ch_sp.push(new_node);
        }

        if !self.ignore_for_row {
            self.current_row_mut()?
                .push(new_node);
        }

        Ok(new_node)
    }

    pub fn dc(&mut self) -> Result<NodeIndex, PatternError> {
        let new_node = self.dc_noskip()?;
        self.skip()?;
        Ok(new_node)
    }

    pub fn dc_noskip(&mut self) -> Result<NodeIndex, PatternError> {
        let new_node = self.graph_mut().add_node(Node::dc(self.current_color));
        self.graph_mut()
            .add_edge(new_node, self.prev, EdgeType::Previous);
        self.graph_mut()
            .add_edge(new_node, self.insert.ok_or(PatternError::NoInsert)?, EdgeType::Insert);

        self.prev = new_node;

        if !self.ignore_for_row {
            self.current_row_mut()?
                .push(new_node);
        }

        Ok(new_node)
    }

    pub fn dec(&mut self) -> Result<NodeIndex, PatternError> {
        let new_node = self.graph_mut().add_node(Node::decrease(self.current_color));
        self.graph_mut()
            .add_edge(new_node, self.prev, EdgeType::Previous);
        self.graph_mut()
            .add_edge(new_node, self.insert.ok_or(PatternError::NoInsert)?, EdgeType::Insert);
        self.skip()?;
        self.graph_mut()
            .add_edge(new_node, self.insert.ok_or(PatternError::NoInsert)?, EdgeType::Insert);
        self.skip()?;

        self.prev = new_node;

        if !self.ignore_for_row {
            self.current_row_mut()?
                .push(new_node);
        }

        Ok(new_node)
    }

    pub fn inc(&mut self) -> Result<(NodeIndex, NodeIndex), PatternError> {
        let s1 = self.dc_noskip()?;
        let s2 = self.dc()?;
        Ok((s1, s2))
    }

    pub fn slip_stitch(&mut self, into: graph::NodeIndex) {
        self.graph_mut().add_edge(self.prev, into, EdgeType::Slip);
    }

    pub fn start_ch_sp(&mut self) -> Result<(), PatternError> {
        if self.current_ch_sp.is_some() {
            return Err(PatternError::NestedChainSpace);
        }

        self.current_ch_sp = Some(vec![self.prev]);

        Ok(())
    }

    pub fn end_ch_sp(&mut self) -> Result<NodeIndex, PatternError> {
        let ch_sp = self
            .current_ch_sp
            .take()
            .ok_or(PatternError::NoChainSpace)?;

        let new_node = self.graph_mut().add_node(Node::ch_sp());
        ch_sp.into_iter().for_each(|neighbour| {
            self.graph_mut()
                .add_edge(new_node, neighbour, EdgeType::Neighbour);
        });

        Ok(new_node)
    }

    pub fn set_ignore(&mut self, ignore: bool) {
        self.ignore_for_row = ignore;
    }

    pub fn change_color(&mut self, color: Vec3) {
        self.current_color = color;
    }
}

pub fn test_pattern_spiral_rounds() -> Result<Pattern, PatternError> {
    let pattern = Pattern::new();
    let mut part = pattern.add_part();

    part.start_ch_sp()?;
    let start = part.prev();
    for _ in 1..=2 {
        part.chain()?;
    }
    part.slip_stitch(start);
    let ch_sp = part.end_ch_sp()?;

    part.new_row()?;
    part.set_insert(ch_sp);
    for _ in 1..=6 {
        part.dc_noskip()?;
    }

    part.new_row()?;
    for _ in 1..=6 {
        part.dc_noskip()?;
        part.dc()?;
    }

    for j in 1..20 {
        part.new_row()?;
        for _ in 1..=6 {
            for _ in 1..=j {
                part.dc()?;
            }
            part.dc_noskip()?;
            part.dc()?;
        }
    }

    Ok(pattern.into_inner())
}

pub fn test_pattern_sphere() -> Result<Pattern, PatternError> {
    let pattern = Pattern::new();
    let mut part = pattern.add_part();

    part.start_ch_sp()?;
    let start = part.prev();
    for _ in 1..=2 {
        part.chain()?;
    }
    part.slip_stitch(start);
    let ch_sp = part.end_ch_sp()?;

    part.new_row()?;
    part.set_insert(ch_sp);
    for _ in 1..=6 {
        part.dc_noskip()?;
    }

    for j in 0..=4 {
        part.new_row()?;
        for _ in 1..=6 {
            for _ in 1..=j {
                part.dc_noskip()?;
                part.skip()?;
            }
            part.dc_noskip()?;
            part.dc()?;
        }
    }

    for _ in 1..=7 {
        part.new_row()?;
        for _ in 1..=36 {
            part.dc()?;
        }
    }

    for j in (0..=4).rev() {
        part.new_row()?;
        for _ in 1..=6 {
            for _ in 1..=j {
                part.dc()?;
            }
            part.dec()?;
        }
    }

    part.new_row()?;
    part.dec()?;
    part.dec()?;

    Ok(pattern.into_inner())
}

pub fn test_pattern_joined_rounds() -> Result<Pattern, PatternError> {
    let pattern = Pattern::new();
    let mut part = pattern.add_part();

    let start = part.prev();
    for _ in 1..=5 {
        part.chain()?;
    }
    part.slip_stitch(start);

    part.turn_noskip()?;
    let start = part.prev();
    part.dc()?;
    for _ in 1..=5 {
        part.inc()?;
    }
    part.slip_stitch(start);

    for round in 1..=20 {
        part.turn()?;
        let start = part.prev();
        for _ in 1..=5 {
            part.inc()?;
            for _ in 1..=round {
                part.dc()?;
            }
        }
        part.inc()?;
        for _ in 1..round {
            part.dc()?;
        }
        part.slip_stitch(start);
    }

    Ok(pattern.into_inner())
}

pub fn test_pattern_flat(n: u32) -> Result<Pattern, PatternError> {
    let pattern = Pattern::new();
    let mut part = pattern.add_part();
    for _ in 1..=n {
        part.chain()?;
    }

    for _ in 1..=n {
        part.turn()?;
        for _ in 1..=n {
            part.dc()?;
        }
    }

    Ok(pattern.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const TEST_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_out");

    #[test]
    fn test_flat() {
        let pattern = test_pattern_flat(7).unwrap();

        let mut file = std::fs::File::create(format!("{TEST_DIR}/flat.dot")).unwrap();
        write!(file, "{}", pattern.to_graphviz()).unwrap();
    }

    #[test]
    fn test_spiral_rounds() {
        let pattern = test_pattern_spiral_rounds().unwrap();

        let mut file = std::fs::File::create(format!("{TEST_DIR}/spiral_rounds.dot")).unwrap();
        write!(file, "{}", pattern.to_graphviz()).unwrap();
    }

    #[test]
    fn test_joined_rounds() {
        let pattern = test_pattern_joined_rounds().unwrap();

        let mut file = std::fs::File::create(format!("{TEST_DIR}/joined_rounds.dot")).unwrap();
        write!(file, "{}", pattern.to_graphviz()).unwrap();
    }

    #[test]
    fn test_sphere() {
        let pattern = test_pattern_sphere().unwrap();

        let mut file = std::fs::File::create(format!("{TEST_DIR}/sphere.dot")).unwrap();
        write!(file, "{}", pattern.to_graphviz()).unwrap();
    }
}
