use serde::{Deserialize, Serialize};

/// A set-off notice consisting of an obligation augmented with a set-off amount.
pub trait SetOff {
    type AccountId;
    type Amount;

    fn new(
        id: Option<usize>,
        debtor: Self::AccountId,
        creditor: Self::AccountId,
        amount: Self::Amount,
        set_off: Self::Amount,
        remainder: Self::Amount,
    ) -> Self;
    fn id(&self) -> Option<usize>;
    fn debtor(&self) -> &Self::AccountId;
    fn creditor(&self) -> &Self::AccountId;
    fn amount(&self) -> Self::Amount;
    fn set_off(&self) -> Self::Amount;
    fn remainder(&self) -> Self::Amount;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SimpleSetoff<AccountId, Amount> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<usize>,
    debtor: AccountId,
    creditor: AccountId,
    amount: Amount,
    set_off: Amount,
    remainder: Amount,
}

impl<AccountId, Amount> SetOff for SimpleSetoff<AccountId, Amount>
where
    Amount: Copy,
{
    type AccountId = AccountId;
    type Amount = Amount;

    fn new(
        id: Option<usize>,
        debtor: Self::AccountId,
        creditor: Self::AccountId,
        amount: Self::Amount,
        set_off: Self::Amount,
        remainder: Self::Amount,
    ) -> Self {
        Self {
            id,
            debtor,
            creditor,
            amount,
            set_off,
            remainder,
        }
    }

    fn id(&self) -> Option<usize> {
        self.id
    }

    fn debtor(&self) -> &Self::AccountId {
        &self.debtor
    }

    fn creditor(&self) -> &Self::AccountId {
        &self.creditor
    }

    fn amount(&self) -> Self::Amount {
        self.amount
    }

    fn set_off(&self) -> Self::Amount {
        self.set_off
    }

    fn remainder(&self) -> Self::Amount {
        self.remainder
    }
}
