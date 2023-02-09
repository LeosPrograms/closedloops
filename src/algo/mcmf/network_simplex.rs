use crate::{FlowPath, Mcmf, Node};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use mcmf::{Capacity, Cost, GraphBuilder, Path, Vertex};

pub struct NetworkSimplex;

impl FlowPath for Path<i32> {
    type Node = i32;
    type Flow = i32;
    type Iter = Vec<i32>;

    fn nodes(&self) -> Self::Iter {
        self.vertices()
            .into_iter()
            .cloned()
            .filter_map(|n| n.as_option())
            .collect()
    }

    fn flow(&self) -> Self::Flow {
        self.flows[0].amount as i32
    }
}

impl From<Node<i32>> for Vertex<i32> {
    fn from(value: Node<i32>) -> Self {
        match value {
            Node::Source => Vertex::Source,
            Node::Sink => Vertex::Sink,
            Node::WithId(id) => Vertex::Node(id),
        }
    }
}

impl Mcmf for NetworkSimplex {
    type AccountId = i32;
    type Amount = i32;
    type Liabilities = BTreeMap<(Node<i32>, Node<i32>), i32>;
    type Error = ();
    type Path = Path<i32>;

    fn mcmf(
        &mut self,
        liabilities: &Self::Liabilities,
    ) -> Result<(Self::Amount, Vec<Self::Path>), Self::Error> {
        // build a graph from given obligation network
        let g = liabilities.iter().fold(
            GraphBuilder::new(),
            |mut acc, ((debtor, creditor), amount)| {
                acc.add_edge(
                    Vertex::<i32>::from(*debtor),
                    Vertex::<i32>::from(*creditor),
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
