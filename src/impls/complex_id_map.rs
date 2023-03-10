use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::id::Id;
use crate::int::Int;
use crate::obligation::{Obligation, SimpleObligation};
use crate::setoff::{SetOff, SimpleSetoff};
use crate::Mtcs;

#[derive(Clone, Debug)]
pub struct ComplexIdMapMtcs<M, O, SO> {
    inner: M,
    _phantom: PhantomData<(O, SO)>,
}

impl<M, O, SO> ComplexIdMapMtcs<M, O, SO> {
    pub fn wrapping(inner: M) -> Self {
        Self {
            inner,
            _phantom: Default::default(),
        }
    }
}

impl<M, O, SO> Mtcs for ComplexIdMapMtcs<M, O, SO>
where
    O: Obligation,
    O::Amount: Int,
    O::AccountId: Id,
    SO: SetOff<Amount = O::Amount, AccountId = O::AccountId>,
    M: Mtcs<
        Obligation = SimpleObligation<usize, O::Amount>,
        SetOff = SimpleSetoff<usize, O::Amount>,
        Obligations = Vec<SimpleObligation<usize, O::Amount>>,
        SetOffs = Vec<SimpleSetoff<usize, O::Amount>>,
    >,
{
    type Obligation = O;
    type SetOff = SO;
    type Obligations = Vec<O>;
    type SetOffs = Vec<SO>;
    type Algo = M::Algo;
    type Error = M::Error;

    fn run(&mut self, obligations: Self::Obligations) -> Result<Self::SetOffs, Self::Error> {
        let firms_mapping: BTreeMap<_, _> = obligations
            .iter()
            .fold(BTreeSet::new(), |mut acc, firm| {
                acc.insert(firm.debtor().clone());
                acc.insert(firm.creditor().clone());
                acc
            })
            .into_iter()
            .enumerate()
            .map(|(idx, firm)| (firm, idx))
            .collect();

        let obligations: Vec<_> = obligations
            .into_iter()
            .map(|o| {
                let debtor = *firms_mapping.get(o.debtor()).unwrap();
                let creditor = *firms_mapping.get(o.creditor()).unwrap();
                SimpleObligation::new(o.id(), debtor, creditor, o.amount()).unwrap()
            })
            .collect();

        let setoffs = self.inner.run(obligations)?;

        let firms_inverse_mapping: BTreeMap<_, _> = firms_mapping
            .into_iter()
            .map(|(firm, idx)| (idx, firm))
            .collect();
        let setoffs = setoffs
            .into_iter()
            .map(|so| {
                let debtor = firms_inverse_mapping.get(so.debtor()).unwrap();
                let creditor = firms_inverse_mapping.get(so.creditor()).unwrap();
                SO::new(
                    so.id(),
                    debtor.clone(),
                    creditor.clone(),
                    so.amount(),
                    so.set_off(),
                    so.remainder(),
                )
            })
            .collect();

        Ok(setoffs)
    }

    fn check(&self, setoffs: &Self::SetOffs) -> Result<(), Self::Error> {
        let firms_mapping: BTreeMap<_, _> = setoffs
            .iter()
            .fold(BTreeSet::new(), |mut acc, firm| {
                acc.insert(firm.debtor().clone());
                acc.insert(firm.creditor().clone());
                acc
            })
            .into_iter()
            .enumerate()
            .map(|(idx, firm)| (firm, idx))
            .collect();

        let setoffs: Vec<_> = setoffs
            .iter()
            .map(|so| {
                let debtor = *firms_mapping.get(so.debtor()).unwrap();
                let creditor = *firms_mapping.get(so.creditor()).unwrap();
                M::SetOff::new(
                    so.id(),
                    debtor,
                    creditor,
                    so.amount(),
                    so.set_off(),
                    so.remainder(),
                )
            })
            .collect();

        self.inner.check(&setoffs)
    }
}
