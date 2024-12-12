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

pub trait SDGCoords:
    Add<Self, Output = Self> + Sub<Self, Output = Self> + Mul<f32, Output = Self> + Sized + Copy
{
    fn random<R: Rng>(rng: &mut R) -> Self;
    fn length(self) -> f32;
    fn is_nan(self) -> bool;
}

impl SDGCoords for Vec2 {
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

impl SDGCoords for Vec3 {
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

#[derive(Debug)]
struct Term {
    start: NodeIndex,
    end: NodeIndex,
    d: f32,
    w: f32,
}

const EPSILON: f32 = 0.01;
const SGD_ITERS: u32 = 10;

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

pub fn sgd<C, N, E>(g: &Graph<N, E>) -> Graph<C, f32, Undirected>
where
    C: SDGCoords,
    E: Into<f32> + Clone,
{
    let mut rng = SmallRng::from_rng(thread_rng()).expect("Couldn't create RNG");

    // turn a crochet graph into pure vertices and edges
    let mut graph = g
        .map(
            |_ix, _node| SDGCoords::random(&mut rng),
            |_ix, edge| edge.clone().into(),
        )
        .into_edge_type::<Undirected>();

    let nodes = graph.node_indices().collect::<Vec<_>>();
    let mut terms = nodes
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
            w: 1.0 / (cost * cost), // the weight w of a term is the inverse square cost.
        })
        .collect::<Vec<_>>();

    terms.shuffle(&mut rng); // each iteration, randomize the list of terms
    let etas: Vec<f32> = schedule(&terms, SGD_ITERS);
    for eta in etas {
        for term in terms.iter_mut() {
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

        terms.shuffle(&mut rng);
    }

    graph
}

const FDG_ITERS: u32 = 10;
const STEP_SIZE: f32 = 0.1;
const ATTRACTIVE_FORCE: f32 = 2.0;
const REPULSIVE_FORCE: f32 = 0.0;

pub fn fdg(g: &mut Graph<Vec3, f32, Undirected>) {
    for _ in 1..=FDG_ITERS {
        let new_pos = g
            .node_references()
            .map(|(n1, p1)| {
                let force: Vec3 = g
                    .node_references()
                    .filter_map(|(n2, p2)| {
                        if n2 == n1 {
                            None
                        } else {
                            let d = p2 - p1;
                            let f = if let Some(e) = g.edges_connecting(n1, n2).next() {
                                ATTRACTIVE_FORCE * f32::log10(d.length() / e.weight())
                            } else {
                                -REPULSIVE_FORCE / d.length_squared()
                            };
                            Some(f * d.normalize())
                        }
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

pub fn normalize(g: &mut Graph<Vec3, f32, Undirected>) {
    let avg_position = g.node_weights().sum::<Vec3>() / g.node_count() as f32;
    let (central, central_pos) = g
        .node_references()
        .min_by(|a, b| {
            (*a.1 - avg_position)
                .length_squared()
                .total_cmp(&(*b.1 - avg_position).length_squared())
        })
        .expect("Couldn't find central node.");

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
        .next_tuple()
        .expect("Node doesn't have at least 3 neighbours");

    let normal = (neighbours.1 - neighbours.0)
        .cross(neighbours.2 - neighbours.0)
        .normalize();
    let transform = Mat4::from_quat(Quat::from_rotation_arc_colinear(normal, Vec3::Y))
        * Mat4::from_translation(-avg_position);

    g.node_weights_mut()
        .for_each(|p| *p = (transform * (p.extend(1.0))).xyz());
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
