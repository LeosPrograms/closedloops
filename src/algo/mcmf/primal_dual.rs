use alloc::collections::BTreeMap;
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;

use num_traits::CheckedAdd;
use petgraph::algo::dijkstra;
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::EdgeFiltered;
use petgraph::Direction;

use crate::algo::max_flow::push_relabel_max_flow;
use crate::id::Id;
use crate::int::Int;
use crate::{MinCostFlow, Node};

#[derive(Debug, Clone, Default)]
pub struct EdgeWeight<Cost, Capacity> {
    pub cost: Cost,         // c
    pub capacity: Capacity, // µ
}

#[derive(Default)]
pub struct PrimalDual<Id, Int>(PhantomData<(Id, Int)>);

impl<N, I> MinCostFlow for PrimalDual<N, I>
where
    N: Id + Copy + Hash,
    I: Int + CheckedAdd,
{
    type NodeWeight = N;
    type EdgeCapacity = I;
    type EdgeCost = I;
    type GraphIter = BTreeMap<(Node<N>, Node<N>), I>;
    type Error = ();
    type Paths = BTreeMap<(N, N), I>;

    fn min_cost_flow(
        &mut self,
        graph_iter: &Self::GraphIter,
    ) -> Result<(Self::EdgeCapacity, Self::Paths), Self::Error> {
        Ok(mtcs_primal_dual::<N, I>(graph_iter))
    }
}

pub fn mtcs_primal_dual<N, I>(
    obligation_list: &BTreeMap<(Node<N>, Node<N>), I>,
) -> (I, BTreeMap<(N, N), I>)
where
    N: Id + Copy + Hash,
    I: Int + CheckedAdd,
{
    let mut graph = DiGraphMap::from_edges(obligation_list.iter().map(|((d, c), capacity)| {
        (
            *d,
            *c,
            EdgeWeight {
                capacity: *capacity,
                cost: I::zero(),
            },
        )
    }));

    let mut max_flow = I::zero();
    let mut paths = BTreeMap::new();

    loop {
        let balance_source: I = graph
            .edges_directed(Node::Source, Direction::Outgoing)
            .map(|(_, _, EdgeWeight { capacity, .. })| *capacity)
            .sum();
        if balance_source == I::zero() {
            // finish if there's no remaining flow
            break;
        }

        // find distance vector i.e. distance from source to every other node.
        // This is a potential place for introducing governance, AKA the 'priority-of-claims'.
        // trick: use a high cost for edges whose capacity is exhausted
        let edge_weights = |(_, _, &EdgeWeight { cost, capacity })| {
            if capacity > I::zero() {
                cost
            } else {
                I::one()
            }
        };
        let distance = dijkstra(&graph, Node::Source, Some(Node::Sink), edge_weights);

        let distance_s_t = distance[&Node::Sink];

        // we define our admissable graph as a subgraph composed of edges that have a
        // `cost <= distance-to-sink` (i.e. `distance_s_t`) and `capacity > 0`
        let admissable_graph = EdgeFiltered::from_fn(&graph, |(_, _, e)| {
            e.cost <= distance_s_t && e.capacity > I::zero()
        });

        let path = push_relabel_max_flow(&admissable_graph, Node::Source, Node::Sink).unwrap();
        let path_flow = path
            .iter()
            .filter_map(|((debtor, _), cap)| (debtor == &Node::Source).then_some(*cap))
            .sum();
        if path_flow == I::zero() {
            break;
        }

        max_flow += path_flow;

        path.into_iter().for_each(|(edge, flow)| {
            let EdgeWeight { capacity, .. } = &mut graph[edge];
            *capacity -= flow;

            if let (Node::WithId(n1), Node::WithId(n2)) = edge {
                *paths.entry((n1, n2)).or_default() += flow;
            }
        });
    }

    (max_flow, paths)
}
