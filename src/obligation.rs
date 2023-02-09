use num_traits::Zero;
use serde::Deserialize;

use crate::error::Error;

pub trait Obligation {
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
pub struct SimpleObligation<AccountId, Amount> {
    id: Option<usize>,
    debtor: AccountId,
    creditor: AccountId,
    amount: Amount,
}

impl<AccountId, Amount> SimpleObligation<AccountId, Amount>
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

impl<AccountId, Amount> Obligation for SimpleObligation<AccountId, Amount>
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

impl<AccountId, Amount> TryFrom<RawObligation<AccountId, Amount>>
    for SimpleObligation<AccountId, Amount>
where
    AccountId: PartialEq,
    Amount: Zero + PartialOrd,
{
    type Error = Error;

    fn try_from(o: RawObligation<AccountId, Amount>) -> Result<Self, Self::Error> {
        Self::new(o.id, o.debtor, o.creditor, o.amount)
    }
}
