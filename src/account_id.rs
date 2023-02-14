use core::fmt::Debug;

/// A trait representing an account identifier.
pub trait AccountId: Clone + Ord + Debug {}

impl AccountId for i32 {}

impl AccountId for u64 {}
