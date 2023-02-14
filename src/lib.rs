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

pub mod algo;
pub mod error;
pub mod id;
pub mod int;
pub mod node;
pub mod obligation;
pub mod setoff;

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::algo::mcmf::MinCostFlow;
use crate::error::Error;
use crate::id::Id;
use crate::int::Int;
use crate::node::Node;
use crate::obligation::Obligation;
use crate::setoff::SetOff;

/// Run the specified clearing algorithm on the given obligation network and generate setoff notices
pub fn run<O, SO, Algo, AccId, Amt>(on: Vec<O>, mut algo: Algo) -> Result<Vec<SO>, Error>
where
    O: Obligation<Amount = Amt, AccountId = AccId>,
    SO: SetOff<Amount = Amt, AccountId = AccId>,
    Algo: MinCostFlow<GraphIter = BTreeMap<(Node<AccId>, Node<AccId>), Amt>, EdgeCapacity = Amt>,
    <Algo as MinCostFlow>::Paths: IntoIterator<Item = ((AccId, AccId), Amt)>,
    AccId: Id,
    Amt: Int,
{
    // calculate the b vector
    let net_position = on.iter().fold(BTreeMap::<_, Amt>::new(), |mut acc, o| {
        *acc.entry(o.creditor()).or_default() += o.amount(); // credit increases the net balance
        *acc.entry(o.debtor()).or_default() -= o.amount(); // debit decreases the net balance
        acc
    });

    let liabilities = on.iter().fold(BTreeMap::new(), |mut acc, o| {
        *acc.entry((o.debtor().into(), o.creditor().into()))
            .or_default() += o.amount();
        acc
    });

    // Add source and sink flows based on values of "b" vector
    let mut liabilities = net_position
        .iter()
        .fold(liabilities, |mut acc, (firm, balance)| {
            match balance.cmp(&Amt::zero()) {
                Ordering::Less => {
                    acc.insert((Node::Source, firm.clone().into()), -*balance);
                }
                Ordering::Greater => {
                    acc.insert((firm.clone().into(), Node::Sink), *balance);
                }
                Ordering::Equal => {}
            }
            acc
        });

    // calculate Net Internal Debt (NID) from the b vector
    let nid: Amt = net_position
        .into_values()
        .filter(|balance| balance > &Amt::zero())
        .sum();

    // calculate total debt
    let td: Amt = on.iter().map(|o| o.amount()).sum();

    // run the (min-cost) max-flow algo
    let (remained, paths) = algo
        .min_cost_flow(&liabilities)
        .map_err(|e| Error::AlgoSpecific(format!("{:?}", e)))?;

    // substract minimum cost maximum flow from the liabilities to get the clearing solution
    let mut tc = td;
    paths.into_iter().for_each(|((n1, n2), amount)| {
        log::trace!("{:?} --> {:?}", n1, n2);

        tc -= amount;
        liabilities
            .entry((n1.into(), n2.into()))
            .and_modify(|e| *e -= amount);
    });

    // Print key results and check for correct sums
    log::info!("----------------------------------");
    log::info!("            NID = {nid:?}");
    log::info!("     Total debt = {td:?}");
    log::info!("Total remainder = {remained:?}");
    log::info!("  Total cleared = {tc:?}");
    // assert_eq!(td, remained + tc);

    // check that b-vec of remainders is all zeros
    let remainders = on.iter().fold(BTreeMap::new(), |mut acc, o| {
        let flow = liabilities
            .get(&(o.debtor().into(), o.creditor().into()))
            .unwrap();
        let remainder = if *flow > o.amount() {
            *flow - o.amount()
        } else {
            Amt::zero()
        };
        if acc.contains_key(&(o.debtor(), o.creditor())) {
            acc.remove(&(o.debtor(), o.creditor()));
        }
        acc.insert((o.debtor(), o.creditor()), remainder);
        acc
    });
    assert!(remainders
        .into_iter()
        .all(|(_, remainder)| remainder == Amt::zero()));

    // Assign cleared amounts to individual obligations
    let setoffs = on
        .into_iter()
        .map(|o| {
            match liabilities
                .get_mut(&(o.debtor().into(), o.creditor().into()))
                .unwrap()
            {
                x if x.is_zero() => SO::new(
                    o.id(),
                    o.debtor(),
                    o.creditor(),
                    o.amount(),
                    Amt::zero(),
                    o.amount(),
                ),
                x if *x < o.amount() => {
                    let oldx = *x;
                    *x = Amt::zero();
                    SO::new(
                        o.id(),
                        o.debtor(),
                        o.creditor(),
                        o.amount(),
                        oldx,
                        o.amount() - oldx,
                    )
                }
                x => {
                    *x -= o.amount();
                    SO::new(
                        o.id(),
                        o.debtor(),
                        o.creditor(),
                        o.amount(),
                        o.amount(),
                        Amt::zero(),
                    )
                }
            }
        })
        .collect();

    Ok(setoffs)
}

/// Check the correctness of the result of the MTCS run (i.e. setoffs)
pub fn check<SO, AccId, Amt>(setoffs: &[SO])
where
    SO: SetOff<AccountId = AccId, Amount = Amt>,
    AccId: Id,
    Amt: Int,
{
    fn assert_eq_pos_neg<AccId, Amt: Int>(b: &BTreeMap<AccId, Amt>) {
        let pos_b: Amt = b
            .values()
            .cloned()
            .filter(|amount| amount > &Amt::zero())
            .sum();

        let neg_b = b
            .values()
            .cloned()
            .filter(|amount| amount < &Amt::zero())
            .sum::<Amt>()
            .neg();

        assert_eq!(pos_b, neg_b);
    }

    // ba - net balance positions of the obligation network
    let ba = setoffs.iter().fold(BTreeMap::<_, _>::new(), |mut acc, so| {
        *acc.entry(so.creditor()).or_default() += so.amount();
        *acc.entry(so.debtor()).or_default() -= so.amount();
        acc
    });

    // bl - net balance positions of the remaining acyclic network
    let bl = setoffs.iter().fold(BTreeMap::<_, _>::new(), |mut acc, so| {
        *acc.entry(so.creditor()).or_default() += so.remainder();
        *acc.entry(so.debtor()).or_default() -= so.remainder();
        acc
    });

    // bc - net balance positions of the cyclic network
    let bc = setoffs.iter().fold(BTreeMap::<_, _>::new(), |mut acc, so| {
        *acc.entry(so.creditor()).or_default() += so.setoff();
        *acc.entry(so.debtor()).or_default() -= so.setoff();
        acc
    });

    // SUM(+NID) == SUM(-NID) for all b-vectors
    assert_eq_pos_neg(&ba);
    assert_eq_pos_neg(&bc);
    assert_eq_pos_neg(&bl);

    // ba == bl
    assert!(ba.iter().all(|(firm, amount)| amount == &bl[firm]));

    // set-off consistency check
    // (i.e. the sum of all set-off amounts where Alice is a debtor equals the sum of all set-off amounts where Alice is a creditor)
    let debtors = setoffs
        .iter()
        .fold(BTreeMap::<_, Amt>::new(), |mut acc, so| {
            *acc.entry(so.debtor()).or_default() += so.setoff();
            acc
        });
    let creditors = setoffs
        .iter()
        .fold(BTreeMap::<_, Amt>::new(), |mut acc, so| {
            *acc.entry(so.creditor()).or_default() += so.setoff();
            acc
        });
    assert!(creditors
        .iter()
        .filter(|(_, amount)| amount > &&Amt::zero())
        .all(|(firm, amount)| amount == &debtors[firm]));
    assert!(debtors
        .iter()
        .filter(|(_, amount)| amount > &&Amt::zero())
        .all(|(firm, amount)| amount == &creditors[firm]));

    let ba_len = ba.len();
    let nid_a: Amt = ba
        .into_values()
        .filter(|amount| amount > &Amt::zero())
        .sum();
    let nid_c: Amt = bc
        .into_values()
        .filter(|amount| amount > &Amt::zero())
        .sum();
    let nid_l: Amt = bl
        .into_values()
        .filter(|amount| amount > &Amt::zero())
        .sum();

    // NID before and after algo run must be the same
    assert_eq!(nid_a, nid_l);

    let debt_before: Amt = setoffs.iter().map(|s| s.amount()).sum();
    let debt_after: Amt = setoffs.iter().map(|s| s.remainder()).sum();
    let compensated: Amt = setoffs.iter().map(|s| s.setoff()).sum();

    log::debug!("num of companies: {ba_len}");
    log::debug!("      NID before: {nid_a}");
    log::debug!(" NID compensated: {nid_c}");
    log::debug!("       NID after: {nid_l}");
    log::debug!("     Debt before: {debt_before}");
    log::debug!(" Debt after + Co: {}", debt_after + compensated);
    log::debug!("         Cleared: {compensated}");
    log::debug!("      Debt after: {debt_after}");
    log::debug!("Debt before - Co: {}", debt_before - compensated);
}
