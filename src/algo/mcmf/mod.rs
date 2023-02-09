pub mod network_simplex;

use core::fmt::Debug;

pub trait MinCostFlow {
    type NodeWeight;
    type EdgeCapacity;
    type EdgeCost;
    type GraphIter;
    type Error: Debug;
    type Paths;

    fn min_cost_flow(
        &mut self,
        graph_iter: &Self::GraphIter,
    ) -> Result<(Self::EdgeCapacity, Self::Paths), Self::Error>;
}
