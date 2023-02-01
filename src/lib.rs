#![no_std]
#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use mcmf::{Capacity, Cost, GraphBuilder, Vertex};
use serde::{Deserialize, Serialize};

//
// Define the Obligation network
//
#[derive(Clone, Debug, Deserialize)]
pub struct Obligation {
    id: Option<i32>,
    debtor: i32,
    creditor: i32,
    amount: i32,
}

#[derive(Clone, Debug, Default)]
pub struct ObligationNetwork {
    pub rows: Vec<Obligation>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SetoffNotice {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i32>,
    debtor: i32,
    creditor: i32,
    amount: i32,
    setoff: i32,
    remainder: i32,
}

pub fn max_flow_network_simplex(on: ObligationNetwork) -> Vec<SetoffNotice> {
    // Calculate the net_position "b" vector as a hashmap
    //          liabilities
    //          and a graph "g"
    // Prepare the clearing as a hashmap
    let mut net_position: BTreeMap<i32, i32> = BTreeMap::new();
    let mut liabilities: BTreeMap<(i32, i32), i32> = BTreeMap::new();
    let mut td: i64 = 0;
    let mut g = GraphBuilder::new();

    let mut clearing = Vec::new();
    for o in on.rows {
        g.add_edge(o.debtor, o.creditor, Capacity(o.amount), Cost(1));
        let balance = net_position.entry(o.debtor).or_insert(0);
        *balance -= o.amount;
        let balance = net_position.entry(o.creditor).or_insert(0);
        *balance += o.amount;
        let liability = liabilities.entry((o.debtor, o.creditor)).or_insert(0);
        *liability += o.amount;
        td += i64::from(o.amount);
        clearing.push((o.id, o.debtor, o.creditor, o.amount));
        // log::debug!("{:?}", o.id);
    }

    // for liability in &clearing {
    //     log::debug!("{:?}", liability);  // Test output
    // }

    // Add source and sink flows based on values of "b" vector
    for (&firm, balance) in &net_position {
        match balance {
            x if x < &0 => g.add_edge(Vertex::Source, firm, Capacity(-balance), Cost(0)),
            x if x > &0 => g.add_edge(firm, Vertex::Sink, Capacity(*balance), Cost(0)),
            &_ => continue,
        };
    }

    // Get the minimum cost maximum flow paths and calculate "nid"
    let (remained, paths) = g.mcmf();
    let nid: i32 = net_position
        .into_values()
        .filter(|balance| balance > &0)
        .sum();

    // substract minimum cost maximum flow from the liabilities to get the clearing solution
    let mut tc: i64 = td;
    for path in paths {
        // print!("{:?} Flow trough: ", path.flows[0].amount);   // Test output
        let _result = path
            .vertices()
            .windows(2)
            .filter(|w| w[0].as_option().is_some() & w[1].as_option().is_some())
            .inspect(|w| {
                // print!("{} --> {} : ", w[0].as_option().unwrap(), w[1].as_option().unwrap());  // Test output
                liabilities
                    .entry((w[0].as_option().unwrap(), w[1].as_option().unwrap()))
                    .and_modify(|e| *e -= i32::try_from(path.flows[0].amount).unwrap());
                tc -= i64::from(path.flows[0].amount);
            })
            .collect::<Vec<_>>();
        // log::debug!();  // Test output
    }

    // for r in &liabilities {
    //     log::debug!("{:?}", r);    // Test output
    // }

    // Print key results and check for correct sums
    log::info!("----------------------------------");
    log::info!("            NID = {nid:?}");
    log::info!("     Total debt = {td:?}");
    log::info!("Total remainder = {remained:?}");
    log::info!("  Total cleared = {tc:?}");
    // assert_eq!(td, remained + tc);

    // Assign cleared amounts to individual obligations
    let mut res = Vec::new();
    for o in clearing {
        // log::debug!("{:?} {:?}", o.0, o.3);     // Test output
        match liabilities.get(&(o.1, o.2)).unwrap() {
            0 => continue,
            x if x < &o.3 => {
                res.push(SetoffNotice {
                    id: o.0,
                    debtor: o.1,
                    creditor: o.2,
                    amount: o.3,
                    setoff: *x,
                    remainder: o.3 - *x,
                });
                liabilities.entry((o.1, o.2)).and_modify(|e| *e = 0);
            }
            _ => {
                liabilities.entry((o.1, o.2)).and_modify(|e| *e -= o.3);
                res.push(SetoffNotice {
                    id: o.0,
                    debtor: o.1,
                    creditor: o.2,
                    amount: 0,
                    setoff: o.3,
                    remainder: 0,
                });
            }
        }
    }
    res
}
