use alloc::vec::Vec;
use core::fmt::Debug;

pub mod mcmf;

pub trait FlowPath {
    type Node;
    type Flow;
    type Iter: IntoIterator<Item = Self::Node>;

    fn nodes(&self) -> Self::Iter;
    fn flow(&self) -> Self::Flow;
}

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
