use crate::{FlowPath, MinCostFlow, Node};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use mcmf::{Capacity, Cost, GraphBuilder, Path, Vertex};

pub type NodeWeight = i32;
pub type EdgeCapacity = i32;

pub struct NetworkSimplex;

impl FlowPath for Path<NodeWeight> {
    type Node = NodeWeight;
    type Flow = EdgeCapacity;
    type Iter = Vec<NodeWeight>;

    fn nodes(&self) -> Self::Iter {
        self.vertices()
            .into_iter()
            .cloned()
            .filter_map(|n| n.as_option())
            .collect()
    }

    fn flow(&self) -> Self::Flow {
        self.flows[0].amount as EdgeCapacity
    }
}

impl From<Node<NodeWeight>> for Vertex<NodeWeight> {
    fn from(value: Node<NodeWeight>) -> Self {
        match value {
            Node::Source => Vertex::Source,
            Node::Sink => Vertex::Sink,
            Node::WithId(id) => Vertex::Node(id),
        }
    }
}

impl MinCostFlow for NetworkSimplex {
    type NodeWeight = NodeWeight;
    type EdgeCapacity = EdgeCapacity;
    type EdgeCost = ();
    type GraphIter = BTreeMap<(Node<NodeWeight>, Node<NodeWeight>), EdgeCapacity>;
    type Error = ();
    type Path = Path<NodeWeight>;

    fn min_cost_flow(
        &mut self,
        graph_iter: &Self::GraphIter,
    ) -> Result<(Self::EdgeCapacity, Vec<Self::Path>), Self::Error> {
        // build a graph from given obligation network
        let g = graph_iter.iter().fold(
            GraphBuilder::new(),
            |mut acc, ((debtor, creditor), amount)| {
                acc.add_edge(
                    Vertex::<NodeWeight>::from(*debtor),
                    Vertex::<NodeWeight>::from(*creditor),
                    Capacity(*amount),
                    Cost(1),
                );
                acc
            },
        );

        // Get the minimum cost maximum flow paths and calculate "nid"
        Ok(g.mcmf())
    }
}
