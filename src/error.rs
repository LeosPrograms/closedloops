use displaydoc::Display;

#[derive(Clone, Display)]
pub enum Error {
    /// Invalid obligation where debtor and creditor are the same
    ObligationToSelf,
    /// Invalid obligation amount, expected positive value
    NonPositiveAmount,
}
