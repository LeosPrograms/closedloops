#[cfg(feature = "lemon-cpp")]
pub mod network_simplex;

pub mod primal_dual;

use core::fmt::Debug;

/// The minimum cost max flow algorithm.
pub trait MinCostFlow {
    type NodeWeight;
    type EdgeCapacity;
    type EdgeCost;
    type GraphIter;
    type Error: Debug;
    type Paths;

    /// Run the algorithm over the specified graph and return the min-cost flow result along with a
    /// list of paths (i.e. edges and flow) that were used.
    fn min_cost_flow(
        &mut self,
        graph_iter: &Self::GraphIter,
    ) -> Result<(Self::EdgeCapacity, Self::Paths), Self::Error>;
}
