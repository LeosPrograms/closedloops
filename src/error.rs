use alloc::string::String;

use displaydoc::Display;

#[derive(Clone, Debug, Display)]
pub enum Error {
    /// Invalid obligation where debtor and creditor are the same
    ObligationToSelf,
    /// Invalid obligation amount, expected positive value
    NonPositiveAmount,
    /// Max flow algorithm specific error
    AlgoSpecific(String),
}
