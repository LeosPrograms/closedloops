pub mod network_simplex;

use alloc::vec::Vec;
use core::fmt::Debug;

use crate::algo::FlowPath;

pub trait MinCostFlow {
    type NodeWeight;
    type EdgeCapacity;
    type EdgeCost;
    type GraphIter;
    type Error: Debug;
    type Path: FlowPath<Flow = Self::EdgeCapacity>;

    fn min_cost_flow(
        &mut self,
        graph_iter: &Self::GraphIter,
    ) -> Result<(Self::EdgeCapacity, Vec<Self::Path>), Self::Error>;
}
