use hooklib::pattern::Pattern;
use petgraph::visit::EdgeRef;
use sgd::sgd;
use crate::model::ModelData;

use super::Vertex;

pub fn model_from_pattern(pattern: &Pattern) -> ModelData {
    let graph = sgd(pattern.graph());

    ModelData {
        vertices: graph
            .node_weights()
            .map(|pos| Vertex::new([pos.x, pos.y, pos.z, 1.]))
            .collect::<Vec<_>>(),
        indices: graph
            .edge_references()
            .flat_map(|e| [e.source().index() as u16, e.target().index() as u16])
            .collect::<Vec<u16>>(),
    }
}