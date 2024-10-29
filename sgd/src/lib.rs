use glam::{Vec2, Vec3};
use petgraph::{algo::dijkstra, stable_graph::NodeIndex, Undirected};
use rand::prelude::*;
use std::ops::{Add, Sub, Mul};

pub trait SDGCoords: Add<Self, Output = Self> + Sub<Self, Output = Self> + Mul<f32, Output = Self> + Sized + Copy {
    fn random() -> Self;
    fn length(self) -> f32;
}

impl SDGCoords for Vec2 {
    fn random() -> Self {
        Vec2::new(
            rand::thread_rng().gen_range(0.0..1.0),
            rand::thread_rng().gen_range(0.0..1.0),
        )
    }

    fn length(self) -> f32 {
        Vec2::length(self)
    }
}

impl SDGCoords for Vec3 {
    fn random() -> Self {
        Vec3::new(
            rand::thread_rng().gen_range(0.0..1.0),
            rand::thread_rng().gen_range(0.0..1.0),
            rand::thread_rng().gen_range(0.0..1.0),
        )
    }

    fn length(self) -> f32 {
        Vec3::length(self)
    }
}

#[derive(Debug)]
struct Term {
    start: NodeIndex,
    end: NodeIndex,
    d: f32,
    w: f32,
}

type Graph<C> = petgraph::Graph<C, f32, Undirected>;

const EPSILON: f32 = 0.01;

fn schedule(terms: &Vec<Term>, t_max: u32) -> Vec<f32> {
    let w_min = terms
        .iter()
        .min_by(|a, b| a.w.partial_cmp(&b.w).expect("tried to compare NaN"))
        .unwrap()
        .w;
    let w_max = terms
        .iter()
        .max_by(|a, b| a.w.partial_cmp(&b.w).expect("tried to compare NaN"))
        .unwrap()
        .w;

    let eta_max = 1.0 / w_min;
    let eta_min = EPSILON / w_max;

    let lambda = f32::ln(eta_max / eta_min) / ((t_max as f32) - 1.0);

    (0..t_max)
        .map(|t| eta_max * f32::exp(-lambda * (t as f32)))
        .collect::<Vec<_>>()
}

pub fn sgd<C, N, E>(g: &petgraph::Graph<N, E>) -> Graph<C> where C: SDGCoords, E: Into<f32> + Clone {
    let mut graph = g
        .filter_map(
            |_ix, _node| {
                Some(SDGCoords::random())
            },
            |_ix, edge| Some(edge.clone().into()),
        )
        .into_edge_type::<Undirected>();

    let nodes = graph.node_indices().collect::<Vec<_>>();
    let mut terms = nodes
        .iter()
        .take(nodes.len() - 1)
        .flat_map(|node| {
            dijkstra::dijkstra(&graph, *node, None, |e| *e.weight())
                .into_iter()
                .map(|(end, cost)| (node, end, cost))
                .collect::<Vec<_>>()
        })
        .filter(|(start, end, _)| *start < end)
        .map(|(start, end, cost)| Term {
            start: *start,
            end,
            d: cost,
            w: 1.0 / (cost * cost),
        })
        .collect::<Vec<_>>();
    terms.shuffle(&mut rand::thread_rng());

    let etas: Vec<f32> = schedule(&terms, 30);
    for eta in etas {
        for term in terms.iter_mut() {
            let mu = f32::min(eta * term.w, 1.0);
            let p_i = graph[term.start];
            let p_j = graph[term.end];

            let d: C = p_i - p_j;
            let mag = d.length();

            // distance constraint
            let r = (mu * (mag - term.d)) / (2.0 * mag);
            let rv = d * r;

            graph[term.start] = p_i - rv;
            graph[term.end] = p_j + rv;
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use hooklib::pattern::*;

    #[test]
    fn test_sgd() {
        let pattern = test_pattern_sphere();

        let graph = sgd::<Vec3, _, _>(pattern.graph());

        for w in graph.node_weights() {
            println!("{}", w);
        }
    }
}
