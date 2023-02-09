use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cmp::Ordering;

use mcmf::{Capacity, Cost, GraphBuilder, Path, Vertex};

use crate::algo::{FlowPath, Mcmf};

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

impl Mcmf for NetworkSimplex {
    type AccountId = i32;
    type Amount = i32;
    type Liabilities = BTreeMap<(i32, i32), i32>;
    type NetPositions = BTreeMap<i32, i32>;
    type Error = ();
    type Path = Path<i32>;

    fn mcmf(
        &mut self,
        liabilities: &Self::Liabilities,
        net_positions: &Self::NetPositions,
    ) -> Result<(Self::Amount, Vec<Self::Path>), Self::Error> {
        // build a graph from given obligation network
        let mut g = liabilities.iter().fold(
            GraphBuilder::new(),
            |mut acc, ((debtor, creditor), amount)| {
                acc.add_edge(*debtor, *creditor, Capacity(*amount), Cost(1));
                acc
            },
        );

        // Add source and sink flows based on values of "b" vector
        net_positions
            .iter()
            .for_each(|(&firm, balance)| match balance.cmp(&0) {
                Ordering::Less => {
                    g.add_edge(Vertex::Source, firm, Capacity(-*balance), Cost(0));
                }
                Ordering::Greater => {
                    g.add_edge(firm, Vertex::Sink, Capacity(*balance), Cost(0));
                }
                Ordering::Equal => {}
            });

        // Get the minimum cost maximum flow paths and calculate "nid"
        Ok(g.mcmf())
    }
}
