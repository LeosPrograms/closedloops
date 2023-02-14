use core::fmt::Debug;

pub trait AccountId: Clone + Ord + Debug {}

impl AccountId for i32 {}
