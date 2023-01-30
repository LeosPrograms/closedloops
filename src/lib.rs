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

mod algo;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

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

pub fn run_algo(on: ObligationNetwork) -> Vec<SetoffNotice> {
    // Calculate the net_position "b" vector as a hashmap
    let net_position = on.rows.iter().fold(BTreeMap::new(), |mut acc, o| {
        *acc.entry(o.debtor).or_insert(0) -= o.amount;
        *acc.entry(o.creditor).or_insert(0) += o.amount;
        acc
    });

    // build a map of liabilities, i.e. (debtor, creditor) v/s amount
    let mut liabilities = on.rows.iter().fold(BTreeMap::new(), |mut acc, o| {
        *acc.entry((o.debtor, o.creditor)).or_insert(0) += o.amount;
        acc
    });

    // calculate total debt
    let td = on.rows.iter().map(|o| o.amount as i64).sum();

    // run the (min-cost) max-flow algo
    let (remained, paths) = algo::mcmf::network_simplex(&on, &net_position);

    let nid: i32 = net_position
        .into_values()
        .filter(|balance| balance > &0)
        .sum();

    // substract minimum cost maximum flow from the liabilities to get the clearing solution
    let mut tc: i64 = td;
    paths.into_iter().for_each(|path| {
        path.vertices()
            .windows(2)
            .filter(|w| w[0].as_option().is_some() & w[1].as_option().is_some())
            .for_each(|w| {
                tc -= i64::from(path.flows[0].amount);
                // print!("{} --> {} : ", w[0].as_option().unwrap(), w[1].as_option().unwrap());  // Test output
                liabilities
                    .entry((w[0].as_option().unwrap(), w[1].as_option().unwrap()))
                    .and_modify(|e| *e -= i32::try_from(path.flows[0].amount).unwrap());
            })
    });

    // Print key results and check for correct sums
    log::info!("----------------------------------");
    log::info!("            NID = {nid:?}");
    log::info!("     Total debt = {td:?}");
    log::info!("Total remainder = {remained:?}");
    log::info!("  Total cleared = {tc:?}");
    // assert_eq!(td, remained + tc);

    // Assign cleared amounts to individual obligations
    on.rows
        .into_iter()
        .flat_map(
            |o| match liabilities.get_mut(&(o.debtor, o.creditor)).unwrap() {
                0 => None,
                x if *x < o.amount => {
                    let oldx = *x;
                    *x = 0;
                    Some(SetoffNotice {
                        id: o.id,
                        debtor: o.debtor,
                        creditor: o.creditor,
                        amount: o.amount,
                        setoff: oldx,
                        remainder: o.amount - oldx,
                    })
                }
                x => {
                    *x -= o.amount;
                    Some(SetoffNotice {
                        id: o.id,
                        debtor: o.debtor,
                        creditor: o.creditor,
                        amount: 0,
                        setoff: o.amount,
                        remainder: 0,
                    })
                }
            },
        )
        .collect()
}
