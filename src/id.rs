use core::fmt::Debug;

/// A trait representing an account identifier.
pub trait Id: Clone + Ord + Debug {}

impl Id for i32 {}

impl Id for u64 {}
