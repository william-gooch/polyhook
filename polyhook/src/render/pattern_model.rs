use crate::render::model::ModelData;
use glam::{Vec2, Vec3};
use hooklib::pattern::{EdgeType, Pattern};
use petgraph::{visit::{Dfs, EdgeFiltered, EdgeRef, NodeRef, Reversed}, Direction::{self, Incoming, Outgoing}};
use sgd::{sgd, SDGCoords};

use super::Vertex;

pub fn model_from_pattern(pattern: &Pattern) -> ModelData {
    let mut graph = sgd::<Vec3, _, _>(&pattern.triangulated_graph());
    sgd::fdg(&mut graph);
    sgd::normalize(&mut graph);

    let graph = pattern.graph()
        .map(
            |ix, node| (graph.raw_nodes()[ix.index()].weight, node), 
            |ix, edge| (graph.raw_edges()[ix.index()].weight, edge), 
        );

    let mut verts: Vec<Vertex> = Vec::new();
    let mut tris: Vec<[u16; 3]> = Vec::new();

    let mut create_rect = |source_pos: Vec3, target_pos: Vec3, width: f32| {
        let dir = target_pos - source_pos;
        let width = dir.cross(source_pos).normalize() * dir.length() * width * 0.5;

        let idx = verts.len() as u16;
        verts.extend([
            Vertex::from((source_pos - width, [1.0, 0.0].into())),
            Vertex::from((source_pos + width, [0.0, 0.0].into())),
            Vertex::from((target_pos + width, [0.0, 1.0].into())),
            Vertex::from((target_pos - width, [1.0, 1.0].into())),
        ].iter());
        tris.push([idx, idx + 1, idx + 2]);
        tris.push([idx + 2, idx + 3, idx]);
    };

    let visit = EdgeFiltered::from_fn(&graph, |e| *e.weight().1 == EdgeType::Previous);
    let visit_rev = Reversed(&visit);
    let mut dfs = Dfs::new(&visit_rev, pattern.start());
    while let Some(node) = dfs.next(&visit_rev) {
        graph.edges_directed(node, Outgoing)
            .for_each(|e| {
                if *e.weight().1 == EdgeType::Insert {
                    let source_pos = graph.node_weight(node).unwrap().0;
                    let target_pos = graph.node_weight(e.target()).unwrap().0;

                    create_rect(source_pos, target_pos, 0.6);
                }
            });

        graph.edges_directed(node, Incoming)
            .for_each(|e| {
                if *e.weight().1 == EdgeType::Previous {
                    let source_pos = graph.node_weight(node).unwrap().0;
                    let target_pos = graph.node_weight(e.source()).unwrap().0;

                    create_rect(source_pos, target_pos, 0.6);
                }
            });
    }

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

pub fn model_from_pattern_2d(pattern: &Pattern) -> ModelData {
    let graph = sgd::<Vec2, _, _>(&pattern.triangulated_graph());

    ModelData::new(
        graph
            .node_weights()
            .map(|pos| Vertex::new([pos.x, pos.y, 0., 1.], [0.0, 0.0]))
            .collect::<Vec<_>>(),
        graph
            .edge_references()
            .flat_map(|e| [e.source().index() as u16, e.target().index() as u16])
            .collect::<Vec<u16>>(),
    )
}
