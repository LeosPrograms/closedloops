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

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use displaydoc::Display;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

//
// Define the Obligation network
//

pub trait ObligationTrait {
    type AccountId;
    type Amount;

    fn id(&self) -> Option<usize>;
    fn debtor(&self) -> Self::AccountId;
    fn creditor(&self) -> Self::AccountId;
    fn amount(&self) -> Self::Amount;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(
    try_from = "RawObligation<AccountId, Amount>",
    bound(deserialize = "AccountId: PartialEq + Deserialize<'de>, \
                    Amount: Zero + PartialOrd + Deserialize<'de>")
)]
pub struct Obligation<AccountId, Amount> {
    id: Option<usize>,
    debtor: AccountId,
    creditor: AccountId,
    amount: Amount,
}

#[derive(Clone, Display)]
pub enum Error {
    /// Invalid obligation where debtor and creditor are the same
    ObligationToSelf,
    /// Invalid obligation amount, expected positive value
    NonPositiveAmount,
}

impl<AccountId, Amount> Obligation<AccountId, Amount>
where
    AccountId: PartialEq,
    Amount: Zero + PartialOrd,
{
    pub fn new(
        id: Option<usize>,
        debtor: AccountId,
        creditor: AccountId,
        amount: Amount,
    ) -> Result<Self, Error> {
        if debtor == creditor {
            Err(Error::ObligationToSelf)
        } else if amount <= Amount::zero() {
            Err(Error::NonPositiveAmount)
        } else {
            Ok(Self {
                id,
                debtor,
                creditor,
                amount,
            })
        }
    }
}

impl<AccountId, Amount> ObligationTrait for Obligation<AccountId, Amount>
where
    AccountId: Copy,
    Amount: Copy,
{
    type AccountId = AccountId;
    type Amount = Amount;

    fn id(&self) -> Option<usize> {
        self.id
    }

    fn debtor(&self) -> Self::AccountId {
        self.debtor
    }

    fn creditor(&self) -> Self::AccountId {
        self.creditor
    }

    fn amount(&self) -> Self::Amount {
        self.amount
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub struct RawObligation<AccountId, Amount> {
    pub id: Option<usize>,
    pub debtor: AccountId,
    pub creditor: AccountId,
    pub amount: Amount,
}

impl<AccountId, Amount> TryFrom<RawObligation<AccountId, Amount>> for Obligation<AccountId, Amount>
where
    AccountId: PartialEq,
    Amount: Zero + PartialOrd,
{
    type Error = Error;

    fn try_from(o: RawObligation<AccountId, Amount>) -> Result<Self, Self::Error> {
        Self::new(o.id, o.debtor, o.creditor, o.amount)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SetoffNotice<AccountId, Amount> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<usize>,
    debtor: AccountId,
    creditor: AccountId,
    amount: Amount,
    setoff: Amount,
    remainder: Amount,
}

pub fn run<'a, O, ON>(on: ON) -> Vec<SetoffNotice<i32, i32>>
where
    O: 'a + ObligationTrait<Amount = i32, AccountId = i32>,
    ON: IntoIterator<Item = &'a O>,
    <ON as IntoIterator>::IntoIter: Clone,
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
    let td = on_iter.clone().map(|o| o.amount() as i64).sum();

    // run the (min-cost) max-flow algo
    let (remained, paths) = algo::mcmf::network_simplex(&liabilities, &net_position);

    // calculate Net Internal Debt (NID) from the b vector
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
                log::trace!(
                    "{} --> {}",
                    w[0].as_option().unwrap(),
                    w[1].as_option().unwrap()
                );

                tc -= i64::from(path.flows[0].amount);
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
            0
        };
        if acc.contains_key(&(o.debtor(), o.creditor())) {
            acc.remove(&(o.debtor(), o.creditor()));
        }
        acc.insert((o.debtor(), o.creditor()), remainder);
        acc
    });
    assert!(remainders.into_iter().all(|(_, remainder)| remainder == 0));

    // Assign cleared amounts to individual obligations
    on_iter
        .clone()
        .flat_map(
            |o| match liabilities.get_mut(&(o.debtor(), o.creditor())).unwrap() {
                0 => None,
                x if *x < o.amount() => {
                    let oldx = *x;
                    *x = 0;
                    Some(SetoffNotice {
                        id: o.id(),
                        debtor: o.debtor(),
                        creditor: o.creditor(),
                        amount: o.amount(),
                        setoff: oldx,
                        remainder: o.amount() - oldx,
                    })
                }
                x => {
                    *x -= o.amount();
                    Some(SetoffNotice {
                        id: o.id(),
                        debtor: o.debtor(),
                        creditor: o.creditor(),
                        amount: 0,
                        setoff: o.amount(),
                        remainder: 0,
                    })
                }
            },
        )
        .collect()
}

pub fn check(setoffs: &[SetoffNotice<i32, i32>]) {
    // ba - net balance positions of the obligation network
    let ba = setoffs.iter().fold(
        BTreeMap::<i32, i32>::new(),
        |mut acc,
         SetoffNotice {
             debtor,
             creditor,
             amount,
             ..
         }| {
            *acc.entry(*creditor).or_default() += *amount;
            *acc.entry(*debtor).or_default() -= *amount;
            acc
        },
    );

    // bl - net balance positions of the remaining acyclic network
    let bl = setoffs.iter().fold(
        BTreeMap::<i32, i32>::new(),
        |mut acc,
         SetoffNotice {
             debtor,
             creditor,
             remainder,
             ..
         }| {
            *acc.entry(*creditor).or_default() += *remainder;
            *acc.entry(*debtor).or_default() -= *remainder;
            acc
        },
    );

    ba.iter().all(|(firm, amount)| amount == &bl[firm]);

    // bc - net balance positions of the cyclic network
    let bc = setoffs.iter().fold(
        BTreeMap::<i32, i32>::new(),
        |mut acc,
         SetoffNotice {
             debtor,
             creditor,
             setoff,
             ..
         }| {
            *acc.entry(*creditor).or_default() += *setoff;
            *acc.entry(*debtor).or_default() -= *setoff;
            acc
        },
    );

    let ba_len = ba.len();
    let nid_a: i32 = ba.into_values().filter(|amount| amount > &0).sum();
    let nid_c: i32 = bc.into_values().filter(|amount| amount > &0).sum();
    let nid_l: i32 = bl.into_values().filter(|amount| amount > &0).sum();

    let debt_before: i32 = setoffs.iter().map(|s| s.amount).sum();
    let debt_after: i32 = setoffs.iter().map(|s| s.setoff).sum();
    let compensated: i32 = setoffs.iter().map(|s| s.remainder).sum();

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
