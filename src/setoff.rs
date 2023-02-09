use serde::Serialize;

pub trait SetOff {
    type AccountId;
    type Amount;

    fn new(
        id: Option<usize>,
        debtor: Self::AccountId,
        creditor: Self::AccountId,
        amount: Self::Amount,
        setoff: Self::Amount,
        remainder: Self::Amount,
    ) -> Self;
    fn id(&self) -> Option<usize>;
    fn debtor(&self) -> Self::AccountId;
    fn creditor(&self) -> Self::AccountId;
    fn amount(&self) -> Self::Amount;
    fn setoff(&self) -> Self::Amount;
    fn remainder(&self) -> Self::Amount;
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct SimpleSetoff<AccountId, Amount> {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<usize>,
    debtor: AccountId,
    creditor: AccountId,
    amount: Amount,
    setoff: Amount,
    remainder: Amount,
}

impl<AccountId, Amount> SetOff for SimpleSetoff<AccountId, Amount>
where
    AccountId: Copy,
    Amount: Copy,
{
    type AccountId = AccountId;
    type Amount = Amount;

    fn new(
        id: Option<usize>,
        debtor: Self::AccountId,
        creditor: Self::AccountId,
        amount: Self::Amount,
        setoff: Self::Amount,
        remainder: Self::Amount,
    ) -> Self {
        Self {
            id,
            debtor,
            creditor,
            amount,
            setoff,
            remainder,
        }
    }

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

    fn setoff(&self) -> Self::Amount {
        self.setoff
    }

    fn remainder(&self) -> Self::Amount {
        self.remainder
    }
}
