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
pub mod amount;
pub mod error;
pub mod obligation;
pub mod setoff;

extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::fmt::Display;

use itertools::Itertools;

use crate::algo::FlowPath;
use crate::algo::Mcmf;
use crate::amount::AmountTrait;
use crate::obligation::ObligationTrait;
use crate::setoff::SetOffNoticeTrait;

pub fn run<'a, O, ON, SO, Algo, AccountId, Amount>(on: ON, mut algo: Algo) -> Vec<SO>
where
    O: 'a + ObligationTrait<Amount = Amount, AccountId = AccountId>,
    ON: IntoIterator<Item = &'a O>,
    <ON as IntoIterator>::IntoIter: Clone,
    SO: SetOffNoticeTrait<Amount = Amount, AccountId = AccountId>,
    Algo: Mcmf<
        Liabilities = BTreeMap<(AccountId, AccountId), Amount>,
        NetPositions = BTreeMap<AccountId, Amount>,
        Amount = Amount,
    >,
    <Algo as Mcmf>::Path: FlowPath<Node = AccountId>,
    AccountId: Ord + Default + Clone + Display,
    Amount: AmountTrait,
{
    let on_iter = on.into_iter();

    // calculate the b vector
    let net_position = on_iter.clone().fold(BTreeMap::new(), |mut acc, o| {
        *acc.entry(o.creditor()).or_default() += o.amount(); // credit increases the net balance
        *acc.entry(o.debtor()).or_default() -= o.amount(); // debit decreases the net balance
        acc
    });

    // create a list of peripheral 'head/tail' nodes (i.e. nodes which are only either creditors or
    // debtors and therefore cannot be part of a cycle.
    let (debtors, creditors) = on_iter.clone().fold(
        (BTreeSet::new(), BTreeSet::new()),
        |(mut debtors, mut creditors), o| {
            debtors.insert(o.debtor());
            creditors.insert(o.creditor());
            (debtors, creditors)
        },
    );
    let peripheral_nodes: Vec<_> = debtors.symmetric_difference(&creditors).collect();

    // build a map of liabilities, i.e. (debtor, creditor) v/s amount, ignoring peripheral nodes and
    // their obligations
    let (heads_tails, liabilities): (Vec<_>, Vec<_>) = on_iter.clone().partition(|o| {
        peripheral_nodes.contains(&&o.debtor()) || peripheral_nodes.contains(&&o.creditor())
    });

    let mut liabilities = liabilities.into_iter().fold(BTreeMap::new(), |mut acc, o| {
        *acc.entry((o.debtor(), o.creditor())).or_default() += o.amount();
        acc
    });

    // calculate total debt
    let td: Amount = on_iter.clone().map(|o| o.amount()).sum();

    // run the (min-cost) max-flow algo
    let (remained, paths) = algo.mcmf(&liabilities, &net_position).unwrap();

    // calculate Net Internal Debt (NID) from the b vector
    let nid: Amount = net_position
        .into_values()
        .filter(|balance| balance > &Amount::default())
        .sum();

    // substract minimum cost maximum flow from the liabilities to get the clearing solution
    let mut tc = td;
    paths.into_iter().for_each(|path| {
        path.nodes()
            .into_iter()
            .tuple_windows()
            .for_each(|(w1, w2)| {
                log::trace!("{} --> {}", w1, w2);

                tc -= path.flow();
                liabilities
                    .entry((w1, w2))
                    .and_modify(|e| *e -= path.flow());
            })
    });

    // Print key results and check for correct sums
    log::info!("----------------------------------");
    log::info!("            NID = {nid:?}");
    log::info!("     Total debt = {td:?}");
    log::info!("Total remainder = {remained:?}");
    log::info!("  Total cleared = {tc:?}");
    // assert_eq!(td, remained + tc);

    // add heads and tails back
    let mut liabilities = heads_tails.into_iter().fold(liabilities, |mut acc, o| {
        *acc.entry((o.debtor(), o.creditor())).or_default() += o.amount();
        acc
    });

    // check that all remainders are zero
    let remainders = on_iter.clone().fold(BTreeMap::new(), |mut acc, o| {
        let flow = liabilities.get(&(o.debtor(), o.creditor())).unwrap();
        let remainder = if *flow > o.amount() {
            *flow - o.amount()
        } else {
            Amount::default()
        };
        if acc.contains_key(&(o.debtor(), o.creditor())) {
            acc.remove(&(o.debtor(), o.creditor()));
        }
        acc.insert((o.debtor(), o.creditor()), remainder);
        acc
    });
    assert!(remainders
        .into_iter()
        .all(|(_, remainder)| remainder == Amount::default()));

    // Assign cleared amounts to individual obligations
    on_iter
        .clone()
        .flat_map(
            |o| match liabilities.get_mut(&(o.debtor(), o.creditor())).unwrap() {
                x if x.is_zero() => None,
                x if *x < o.amount() => {
                    let oldx = *x;
                    *x = Amount::default();
                    Some(SO::new(
                        o.id(),
                        o.debtor(),
                        o.creditor(),
                        o.amount(),
                        oldx,
                        o.amount() - oldx,
                    ))
                }
                x => {
                    *x -= o.amount();
                    Some(SO::new(
                        o.id(),
                        o.debtor(),
                        o.creditor(),
                        Amount::zero(),
                        o.amount(),
                        Amount::zero(),
                    ))
                }
            },
        )
        .collect()
}

pub fn check<SO, AccountId, Amount>(setoffs: &[SO])
where
    SO: SetOffNoticeTrait<AccountId = AccountId, Amount = Amount>,
    AccountId: Ord + Default + Clone + Display,
    Amount: AmountTrait,
{
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

    ba.iter().all(|(firm, amount)| amount == &bl[firm]);

    // bc - net balance positions of the cyclic network
    let bc = setoffs.iter().fold(BTreeMap::<_, _>::new(), |mut acc, so| {
        *acc.entry(so.creditor()).or_default() += so.setoff();
        *acc.entry(so.debtor()).or_default() -= so.setoff();
        acc
    });

    let ba_len = ba.len();
    let nid_a: Amount = ba
        .into_values()
        .filter(|amount| amount > &Amount::zero())
        .sum();
    let nid_c: Amount = bc
        .into_values()
        .filter(|amount| amount > &Amount::zero())
        .sum();
    let nid_l: Amount = bl
        .into_values()
        .filter(|amount| amount > &Amount::zero())
        .sum();

    let debt_before: Amount = setoffs.iter().map(|s| s.amount()).sum();
    let debt_after: Amount = setoffs.iter().map(|s| s.setoff()).sum();
    let compensated: Amount = setoffs.iter().map(|s| s.remainder()).sum();

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
