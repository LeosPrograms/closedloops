use alloc::vec::Vec;
use core::fmt::Debug;

use crate::algo::FlowPath;

pub mod network_simplex;

pub trait Mcmf {
    type AccountId;
    type Amount;
    type Liabilities;
    type Error: Debug;
    type Path: FlowPath<Flow = Self::Amount>;

    fn mcmf(
        &mut self,
        liabilities: &Self::Liabilities,
    ) -> Result<(Self::Amount, Vec<Self::Path>), Self::Error>;
}
