use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cmp::Ordering;

use mcmf::{Capacity, Cost, GraphBuilder, Path, Vertex};

use crate::ObligationNetwork;

pub(crate) fn network_simplex(
    on: &ObligationNetwork,
    net_position: &BTreeMap<i32, i32>,
) -> (i32, Vec<Path<i32>>) {
    // build a graph from given obligation network
    let mut g = on.rows.iter().fold(GraphBuilder::new(), |mut acc, o| {
        acc.add_edge(o.debtor, o.creditor, Capacity(o.amount), Cost(1));
        acc
    });

    // Add source and sink flows based on values of "b" vector
    net_position
        .iter()
        .for_each(|(&firm, balance)| match balance.cmp(&0) {
            Ordering::Less => {
                g.add_edge(Vertex::Source, firm, Capacity(-balance), Cost(0));
            }
            Ordering::Greater => {
                g.add_edge(firm, Vertex::Sink, Capacity(*balance), Cost(0));
            }
            Ordering::Equal => {}
        });

    // Get the minimum cost maximum flow paths and calculate "nid"
    g.mcmf()
}
