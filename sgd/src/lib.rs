use glam::{Mat4, Quat, Vec2, Vec3, Vec4Swizzles};
use itertools::Itertools;
use petgraph::{
    algo::dijkstra,
    graph::NodeIndex,
    visit::{EdgeRef, IntoNodeReferences},
    Undirected,
};
use rand::prelude::*;
use std::ops::{Add, Mul, Sub};

pub trait SDGCoords:
    Add<Self, Output = Self> + Sub<Self, Output = Self> + Mul<f32, Output = Self> + Sized + Copy
{
    fn random<R: Rng>(rng: &mut R) -> Self;
    fn length(self) -> f32;
}

impl SDGCoords for Vec2 {
    fn random<R: Rng>(rng: &mut R) -> Self {
        Vec2::new(rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0))
    }

    fn length(self) -> f32 {
        Vec2::length(self)
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

pub fn sgd<C, N, E>(g: &petgraph::Graph<N, E>) -> Graph<C>
where
    C: SDGCoords,
    E: Into<f32> + Clone,
{
    let mut rng = SmallRng::from_rng(thread_rng()).expect("Couldn't create RNG");

    // turn a crochet graph into pure vertices and edges
    let mut graph = g
        .filter_map(
            |_ix, _node| Some(SDGCoords::random(&mut rng)),
            |_ix, edge| Some(edge.clone().into()),
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
    let etas: Vec<f32> = schedule(&terms, 30);
    for eta in etas {
        for term in terms.iter_mut() {
            let mu = f32::min(eta * term.w, 1.0); // limit the step size to at most 1.
            let p_i = graph[term.start];
            let p_j = graph[term.end];

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

pub fn normalize(g: &mut Graph<Vec3>) {
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
        .map(|n| central_pos - g.node_weight(n).unwrap())
        .take(3)
        .next_tuple()
        .expect("Node doesn't have at least 3 neighbours");

    let normal = (neighbours.1 - neighbours.0)
        .cross(neighbours.2 - neighbours.0)
        .normalize();
    let transform = Mat4::from_quat(Quat::from_rotation_arc_colinear(normal, Vec3::Y))
        * Mat4::from_translation(-central_pos);

    g.node_weights_mut()
        .for_each(|p| *p = (transform * (p.extend(1.0))).xyz());
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

    #[test]
    fn test_sgd_size() {
        for i in (5..=30).step_by(5) {
            let pattern = test_pattern_flat(i);
            let _ = sgd::<Vec3, _, _>(&pattern.triangulated_graph());
        }
    }
}
