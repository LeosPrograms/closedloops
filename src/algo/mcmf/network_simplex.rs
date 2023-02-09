use alloc::collections::BTreeMap;

use mcmf::{Capacity, Cost, GraphBuilder, Vertex};

use crate::{MinCostFlow, Node};

pub type NodeWeight = i32;
pub type EdgeCapacity = i32;

pub struct NetworkSimplex;

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
    type Paths = BTreeMap<(NodeWeight, NodeWeight), EdgeCapacity>;

    fn min_cost_flow(
        &mut self,
        graph_iter: &Self::GraphIter,
    ) -> Result<(Self::EdgeCapacity, Self::Paths), Self::Error> {
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
        let (max_flow, paths) = g.mcmf();
        let paths = paths.into_iter().fold(BTreeMap::new(), |mut acc, p| {
            p.flows
                .into_iter()
                .filter_map(|f| {
                    if let (Some(n1), Some(n2)) = (f.a.as_option(), f.b.as_option()) {
                        Some(((n1, n2), f.amount as EdgeCapacity))
                    } else {
                        None
                    }
                })
                .for_each(|((n1, n2), flow)| {
                    *acc.entry((n1, n2)).or_default() += flow;
                });
            acc
        });

        Ok((max_flow, paths))
    }
}
