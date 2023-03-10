use alloc::{vec, vec::Vec};
use core::marker::PhantomData;

use crate::int::Int;
use crate::obligation::{Obligation, RawObligation, SimpleObligation};
use crate::setoff::{SetOff, SimpleSetoff};
use crate::Mtcs;

#[derive(Clone, Debug)]
pub struct ComplexIdMtcs<M, O, SO> {
    inner: M,
    _phantom: PhantomData<(O, SO)>,
}

impl<M, O, SO> ComplexIdMtcs<M, O, SO> {
    pub fn wrapping(inner: M) -> Self {
        Self {
            inner,
            _phantom: Default::default(),
        }
    }

    fn firm_pos<Id: PartialEq>(firms: &mut Vec<Id>, firm: Id) -> usize {
        if let Some(pos) = firms.iter().position(|f| *f == firm) {
            pos
        } else {
            firms.push(firm);
            firms.len() - 1
        }
    }
}

impl<M, O, SO> Mtcs for ComplexIdMtcs<M, O, SO>
where
    O: Obligation + Into<RawObligation<O::AccountId, O::Amount>>,
    O::Amount: Int,
    O::AccountId: PartialEq + Clone,
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
        let mut firms = vec![];
        let obligations: Vec<_> = obligations
            .into_iter()
            .map(Into::into)
            .map(|o| {
                let debtor = Self::firm_pos(&mut firms, o.debtor);
                let creditor = Self::firm_pos(&mut firms, o.creditor);
                SimpleObligation::new(o.id, debtor, creditor, o.amount).unwrap()
            })
            .collect();

        let setoffs = self.inner.run(obligations)?;
        let setoffs = setoffs
            .into_iter()
            .map(|so| {
                SO::new(
                    so.id,
                    firms[so.debtor].clone(),
                    firms[so.creditor].clone(),
                    so.amount,
                    so.set_off,
                    so.remainder,
                )
            })
            .collect();

        Ok(setoffs)
    }

    fn check(&self, setoffs: &Self::SetOffs) -> Result<(), Self::Error> {
        let mut firms = vec![];
        let setoffs: Vec<_> = setoffs
            .iter()
            .map(|so| {
                let debtor = Self::firm_pos(&mut firms, so.debtor().clone());
                let creditor = Self::firm_pos(&mut firms, so.creditor().clone());

                SimpleSetoff::new(
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
