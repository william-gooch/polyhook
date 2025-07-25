use crate::render::model::ModelData;
use glam::{Vec2, Vec3};
use hooklib::pattern::{EdgeType, Pattern};
use petgraph::{
    visit::{EdgeRef, IntoNodeReferences},
    Direction::{Incoming, Outgoing},
};
use sgd::sgd;

use super::Vertex;

fn model_from_graph(
    graph: petgraph::Graph<(Vec3, &hooklib::pattern::Node), (f32, &EdgeType)>,
) -> ModelData {
    let mut verts: Vec<Vertex> = Vec::new();
    let mut tris: Vec<[u16; 3]> = Vec::new();

    let mut create_rect =
        |source_pos: Vec3, target_pos: Vec3, tangent: Vec3, width: f32, color: Vec3| {
            let dir = target_pos - source_pos;
            let offset_len = width * 0.5;

            let normal = dir.cross(tangent).normalize();
            let offset_x = normal.cross(dir).normalize() * offset_len;
            // let offset_x = tangent.normalize() * offset_len;

            let idx = verts.len() as u16;
            verts.extend(
                [
                    Vertex::new(
                        source_pos - offset_x,
                        [1.0, 0.0].into(),
                        color,
                        normal,
                        tangent,
                    ),
                    Vertex::new(
                        source_pos + offset_x,
                        [0.0, 0.0].into(),
                        color,
                        normal,
                        tangent,
                    ),
                    Vertex::new(
                        target_pos + offset_x,
                        [0.0, 0.5].into(),
                        color,
                        normal,
                        tangent,
                    ),
                    Vertex::new(
                        target_pos - offset_x,
                        [1.0, 0.5].into(),
                        color,
                        normal,
                        tangent,
                    ),
                ]
                .iter(),
            );
            tris.push([idx, idx + 1, idx + 2]);
            tris.push([idx + 2, idx + 3, idx]);

            let idx = verts.len() as u16;
            verts.extend(
                [
                    Vertex::new(
                        source_pos - offset_x,
                        [0.0, 0.5].into(),
                        color,
                        -normal,
                        tangent,
                    ),
                    Vertex::new(
                        source_pos + offset_x,
                        [1.0, 0.5].into(),
                        color,
                        -normal,
                        tangent,
                    ),
                    Vertex::new(
                        target_pos + offset_x,
                        [1.0, 1.0].into(),
                        color,
                        -normal,
                        tangent,
                    ),
                    Vertex::new(
                        target_pos - offset_x,
                        [0.0, 1.0].into(),
                        color,
                        -normal,
                        tangent,
                    ),
                ]
                .iter(),
            );
            tris.push([idx, idx + 2, idx + 1]);
            tris.push([idx + 3, idx + 2, idx]);
        };

    graph
        .node_references()
        .for_each(|(node, (source_pos, node_type))| {
            let color = match node_type {
                hooklib::pattern::Node::Stitch { color, .. } => *color,
                _ => Vec3::ONE,
            };
            graph.edges_directed(node, Outgoing).for_each(|e| {
                if *e.weight().1 == EdgeType::Insert {
                    let target_pos = graph.node_weight(e.target()).unwrap().0;

                    let tangent_1 = graph
                        .edges_directed(node, Incoming)
                        .find(|e| *e.weight().1 == EdgeType::Previous)
                        .map(|e| source_pos - graph.node_weight(e.source()).unwrap().0)
                        .unwrap_or(Vec3::X);
                    let tangent_2 = graph
                        .edges_directed(node, Outgoing)
                        .find(|e| *e.weight().1 == EdgeType::Previous)
                        .map(|e| source_pos - graph.node_weight(e.target()).unwrap().0)
                        .unwrap_or(Vec3::X);
                    let tangent = if tangent_1.dot(tangent_2) <= 0.0 {
                        (tangent_1 - tangent_2) / 2.0
                    } else {
                        (tangent_1 + tangent_2) / 2.0
                    };

                    create_rect(*source_pos, target_pos, tangent, tangent.length(), color);
                } else if *e.weight().1 == EdgeType::Previous && node_type.stitch_type() == "ch" {
                    let target_pos = graph.node_weight(e.target()).unwrap().0;

                    let tangent_1 = graph
                        .edges_directed(node, Incoming)
                        .find(|e| *e.weight().1 == EdgeType::Insert)
                        .map(|e| source_pos - graph.node_weight(e.source()).unwrap().0)
                        .unwrap_or(Vec3::X);
                    let tangent_2 = graph
                        .edges_directed(node, Incoming)
                        .find(|e| *e.weight().1 == EdgeType::Previous)
                        .map(|e| source_pos - graph.node_weight(e.source()).unwrap().0)
                        .unwrap_or(Vec3::X);
                    let tangent = if tangent_1.dot(tangent_2) <= 0.0 {
                        (tangent_1 - tangent_2) / 2.0
                    } else {
                        (tangent_1 + tangent_2) / 2.0
                    };

                    create_rect(*source_pos, target_pos, tangent, tangent.length(), color);
                }
            });
        });

    // graph.edges_directed(node, Incoming)
    //     .for_each(|e| {
    //         if *e.weight().1 == EdgeType::Previous {
    //             let source_pos = graph.node_weight(node).unwrap().0;
    //             let target_pos = graph.node_weight(e.source()).unwrap().0;

    //             create_rect(source_pos, target_pos, 0.6);
    //         }
    //     });

    ModelData::new(
        // vertices: graph
        //     .node_weights()
        //     .map(|(pos, _node)| Vertex::new([pos.x, pos.y, pos.z, 1.]))
        //     .collect::<Vec<_>>(),
        // indices: graph
        //     .edge_references()
        //     .flat_map(|e| [e.source().index() as u16, e.target().index() as u16])
        //     .collect::<Vec<u16>>(),
        verts.into_iter().collect::<Vec<Vertex>>(),
        tris.into_iter().flatten().collect::<Vec<u16>>(),
    )
}

pub fn model_from_pattern(pattern: &Pattern) -> ModelData {
    println!("Number of nodes: {}", pattern.graph().node_count());
    let start_time = std::time::Instant::now();
    let mut graph = sgd::<Vec3, _, _>(&pattern.triangulated_graph());
    println!("SGD took {}s", start_time.elapsed().as_secs_f32());
    sgd::fdg(&mut graph);
    println!("FDG took {}s", start_time.elapsed().as_secs_f32());
    let _ = sgd::normalize(&mut graph);
    println!("Norm took {}s", start_time.elapsed().as_secs_f32());

    let orig_graph = pattern.graph();
    let graph = orig_graph.map(
        |ix, node| (graph.raw_nodes()[ix.index()].weight, node),
        |ix, edge| (graph.raw_edges()[ix.index()].weight, edge),
    );

    model_from_graph(graph)
}

pub fn model_from_pattern_2d(pattern: &Pattern) -> ModelData {
    let graph = sgd::<Vec2, _, _>(&pattern.triangulated_graph());

    let orig_graph = pattern.graph();
    let graph = orig_graph.map(
        |ix, node| {
            let p = graph.raw_nodes()[ix.index()].weight;
            ([p.x, p.y, 0.0].into(), node)
        },
        |ix, edge| (graph.raw_edges()[ix.index()].weight, edge),
    );

    model_from_graph(graph)
}

#[cfg(test)]
mod tests {
    use hooklib::pattern::test_pattern_flat;

    use super::*;
    use std::io::Write;

    const TEST_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test_out");

    #[test]
    fn test_runtime() {
        // Requirement: a pattern of less than 2500 nodes shouldn't take more than 30s to compute.

        // Flat pattern should have at least 49 * 49 = 2401 nodes
        // (more including foundation chain)
        let pattern = test_pattern_flat(49).unwrap();
        assert!(pattern.graph().node_count() <= 2500);

        let start_time = std::time::Instant::now();
        let _model = model_from_pattern(&pattern);
        let elapsed = start_time.elapsed().as_secs_f64();

        assert!(elapsed <= 30.0);
    }

    #[test]
    #[ignore = "Analyzes the runtime for many different graph sizes, takes a few minutes to run."]
    fn test_analyze_runtime() {
        // Requirement: a pattern of less than 2500 nodes shouldn't take more than 30s to compute.

        // Flat pattern should have at least 49 * 49 = 2401 nodes
        // (more including foundation chain)

        let mut file = std::fs::File::create(format!("{TEST_DIR}/runtime.csv")).unwrap();
        (5..49).for_each(|i| {
            let pattern = test_pattern_flat(i).unwrap();
            let nodes = pattern.graph().node_count();

            let start_time = std::time::Instant::now();
            let _model = model_from_pattern(&pattern);
            let elapsed = start_time.elapsed().as_secs_f64();

            writeln!(file, "{nodes}, {elapsed}").unwrap()
        });
    }
}
