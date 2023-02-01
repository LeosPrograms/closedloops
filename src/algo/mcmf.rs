use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cmp::Ordering;

use mcmf::{Capacity, Cost, GraphBuilder, Path, Vertex};

pub(crate) fn network_simplex(
    liabilities: &BTreeMap<(i32, i32), i32>,
    net_position: &BTreeMap<i32, i32>,
) -> (i32, Vec<Path<i32>>) {
    // build a graph from given obligation network
    let mut g = liabilities.iter().fold(
        GraphBuilder::new(),
        |mut acc, ((debtor, creditor), amount)| {
            acc.add_edge(*debtor, *creditor, Capacity(*amount), Cost(1));
            acc
        },
    );

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
