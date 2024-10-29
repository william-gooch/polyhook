use rand::prelude::*;
use itertools::Itertools;
use glam::Vec3;
use petgraph::{stable_graph::NodeIndex, Undirected, algo::dijkstra};

#[derive(Debug)]
struct Term {
    start: NodeIndex,
    end: NodeIndex,
    d: f32,
    w: f32,
}

type Graph = petgraph::Graph<Vec3, f32, Undirected>;

const EPSILON: f32 = 0.01;

fn schedule(terms: &Vec<Term>, t_max: u32) -> Vec<f32> {
    let w_min = terms.iter().min_by(|a, b| a.w.partial_cmp(&b.w).expect("tried to compare NaN")).unwrap().w;
    let w_max = terms.iter().max_by(|a, b| a.w.partial_cmp(&b.w).expect("tried to compare NaN")).unwrap().w;

    let eta_max = 1.0 / w_min;
    let eta_min = EPSILON / w_max;

    let lambda = f32::ln(eta_max / eta_min) / ((t_max as f32) - 1.0);

    (0..t_max)
        .map(|t| eta_max * f32::exp(-lambda * (t as f32)))
        .collect::<Vec<_>>()
}

pub fn sgd<N, E: Into<f32> + Clone>(g: &petgraph::Graph<N, E>) -> Graph {
    let mut graph = g.filter_map(
        |_ix, _node| Some(Vec3::new(rand::thread_rng().gen_range(0.0..1.0), rand::thread_rng().gen_range(0.0..1.0), rand::thread_rng().gen_range(0.0..1.0))),
        |_ix, edge| Some(edge.clone().into())
    ).into_edge_type::<Undirected>();

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
        .filter(|(start, end, cost)| *start < end)
        .map(|(start, end, cost)| Term { start: *start, end, d: cost, w: 1.0 / (cost * cost) })
        .collect::<Vec<_>>();
    terms.shuffle(&mut rand::thread_rng());

    let etas: Vec<f32> = schedule(&terms, 30);
    for eta in etas {
        for term in terms.iter_mut() {
            let mu = f32::min(eta * term.w, 1.0);
            let p_i = graph[term.start];
            let p_j = graph[term.end];

            let d = p_i - p_j;
            let mag = d.length();

            // distance constraint
            let r = (mu * (mag - term.d)) / (2.0 * mag);
            let rv = r * d;
            
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

        let graph = sgd(pattern.graph());

        for w in graph.node_weights() {
            println!("{}", w);
        }
    }
}
