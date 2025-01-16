//! Stochastic gradient descent (SGD) graph layout implementation for Rust.

use glam::{Mat4, Quat, Vec2, Vec3, Vec4Swizzles};
use itertools::Itertools;
use petgraph::{
    algo::dijkstra,
    graph::NodeIndex,
    visit::{EdgeRef, IntoNodeReferences},
    Graph, Undirected,
};
use rand::prelude::*;
use std::ops::{Add, Mul, Sub};

/// Trait for types that can be used as coordinates for SGD.
pub trait SGDCoords:
    Add<Self, Output = Self> + Sub<Self, Output = Self> + Mul<f32, Output = Self> + Sized + Copy
{
    fn random<R: Rng>(rng: &mut R) -> Self;
    fn length(self) -> f32;
    fn is_nan(self) -> bool;
}

impl SGDCoords for Vec2 {
    fn random<R: Rng>(rng: &mut R) -> Self {
        Vec2::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0))
    }

    fn length(self) -> f32 {
        Vec2::length(self)
    }

    fn is_nan(self) -> bool {
        Vec2::is_nan(self)
    }
}

impl SGDCoords for Vec3 {
    fn random<R: Rng>(rng: &mut R) -> Self {
        Vec3::new(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        )
    }

    fn length(self) -> f32 {
        Vec3::length(self)
    }

    fn is_nan(self) -> bool {
        Vec3::is_nan(self)
    }
}

/// A single term in the SGD process, created from a shortest-path between two nodes.
#[derive(Debug)]
struct Term {
    start: NodeIndex,
    end: NodeIndex,
    d: f32,
    w: f32,
}

const EPSILON: f32 = 0.01;
const SGD_ITERS: u32 = 10;

/// Produces the annealing schedule used for SGD.
fn schedule(terms: &[Term], t_max: u32) -> Vec<f32> {
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

/// Performs SGD on the given graph, converting all nodes to the given [`SGDCoords`] type, and all edges into their length given by [`Into<f32>`].
pub fn sgd<C, N, E>(g: &Graph<N, E>) -> Graph<C, f32, Undirected>
where
    C: SGDCoords,
    E: Into<f32> + Clone,
{
    let mut rng = SmallRng::from_rng(thread_rng()).expect("Couldn't create RNG");

    // turn a crochet graph into pure vertices and edges
    let mut graph = g
        .map(
            |_ix, _node| SGDCoords::random(&mut rng),
            |_ix, edge| edge.clone().into(),
        )
        .into_edge_type::<Undirected>();

    let nodes = graph.node_indices().collect::<Vec<_>>();
    let terms = nodes
        .iter()
        .take(nodes.len() - 1) // ignore the last node because it'll be covered by all the others
        .flat_map(|node| {
            // find the shortest path from each node to each other node
            dijkstra::dijkstra(&graph, *node, None, |e| *e.weight())
                .into_iter()
                .map(|(end, cost)| (node, end, cost))
                .collect::<Vec<_>>()
        })
        .filter(|(start, end, _)| *start < end) // and only count the unique paths
        .map(|(&start, end, cost)| Term {
            start,
            end,
            d: cost,
            w: f32::powi(cost, -2), // the weight w of a term is the inverse square cost.
        })
        .collect::<Vec<_>>();

    if terms.is_empty() {
        eprintln!("No terms found in the graph! Was there more than one node?");
        return graph;
    }

    // shuffle the terms twice and alternate between both shuffles
    let mut terms_order_1 = (0..terms.len()).collect::<Vec<_>>();
    terms_order_1.shuffle(&mut rng);
    let mut terms_order_2 = (0..terms.len()).collect::<Vec<_>>();
    terms_order_2.shuffle(&mut rng);
    let mut terms_orders = { std::iter::repeat([&terms_order_1, &terms_order_2]).flatten() };

    let etas: Vec<f32> = schedule(&terms, SGD_ITERS);
    for eta in etas {
        let term_idxs = terms_orders.next().unwrap();
        for &term_idx in term_idxs.iter() {
            let term = &terms[term_idx];
            let mu = f32::min(eta * term.w, 1.0); // limit the step size to at most 1.
            let p_i: C = graph[term.start];
            let p_j: C = graph[term.end];

            let d: C = p_i - p_j;
            let mag = d.length();

            let r = (mu * (mag - term.d)) / (2.0 * mag);
            let rv = d * r;

            graph[term.start] = p_i - rv;
            graph[term.end] = p_j + rv;
        }
    }

    graph
}

const FDG_ITERS: u32 = 10;
const STEP_SIZE: f32 = 0.1;
const ATTRACTIVE_FORCE: f32 = 2.0;

/// Perform force-directed graph layout on a graph.
/// Uses the Tutte approach - attractive forces and no repulsive forces.
pub fn fdg(g: &mut Graph<Vec3, f32, Undirected>) {
    for _ in 1..=FDG_ITERS {
        let new_pos = g
            .node_references()
            .map(|(n1, p1)| {
                let force: Vec3 = g
                    .edges(n1)
                    .map(|e| {
                        let n2 = if e.source() == n1 {
                            e.target()
                        } else {
                            e.source()
                        };
                        let p2 = g.node_weight(n2).unwrap();
                        let d = p2 - p1;
                        let f = ATTRACTIVE_FORCE * f32::log10(d.length() / e.weight());
                        f * d.normalize()
                    })
                    .sum();
                (n1, g.node_weight(n1).unwrap() + (STEP_SIZE * force))
            })
            .collect::<Vec<_>>();

        new_pos.into_iter().for_each(|(n, p)| {
            let w = g.node_weight_mut(n).unwrap();
            *w = p;
        });
    }
}

/// Normalize a graph to be roughly in the same position each time, regardless of initial random state.
pub fn normalize(g: &mut Graph<Vec3, f32, Undirected>) -> Option<()> {
    let avg_position = g.node_weights().sum::<Vec3>() / g.node_count() as f32;
    let (central, central_pos) = g.node_references().min_by(|a, b| {
        (*a.1 - avg_position)
            .length_squared()
            .total_cmp(&(*b.1 - avg_position).length_squared())
    })?;

    let neighbours: (Vec3, Vec3, Vec3) = g
        .edges(central)
        .map(|e| {
            if e.source() == central {
                e.target()
            } else {
                e.source()
            }
        })
        .unique()
        .map(|n| central_pos - g.node_weight(n).unwrap())
        .take(3)
        .next_tuple()?;

    let normal = (neighbours.1 - neighbours.0)
        .cross(neighbours.2 - neighbours.0)
        .normalize();
    let transform = Mat4::from_quat(Quat::from_rotation_arc_colinear(normal, Vec3::Y))
        * Mat4::from_translation(-avg_position);

    g.node_weights_mut()
        .for_each(|p| *p = (transform * (p.extend(1.0))).xyz());

    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hooklib::pattern::*;

    #[test]
    fn test_sgd() {
        let pattern = test_pattern_sphere().unwrap();

        let graph = sgd::<Vec3, _, _>(&*pattern.graph());

        for w in graph.node_weights() {
            println!("{}", w);
        }
    }

    #[test]
    fn test_sgd_size() {
        for i in (5..=30).step_by(5) {
            let pattern = test_pattern_flat(i).unwrap();
            let _ = sgd::<Vec3, _, _>(&pattern.triangulated_graph());
        }
    }
}
