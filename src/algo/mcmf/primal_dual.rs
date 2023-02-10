use alloc::collections::BTreeMap;
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;

use num_traits::CheckedAdd;
use petgraph::algo::dijkstra;
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::EdgeFiltered;
use petgraph::Direction;

use crate::account_id::AccountId;
use crate::algo::max_flow::push_relabel_max_flow;
use crate::amount::Amount;
use crate::{MinCostFlow, Node};

#[derive(Debug, Clone, Default)]
pub struct EdgeWeight<Cost, Capacity> {
    pub cost: Cost,         // c
    pub capacity: Capacity, // µ
}

#[derive(Default)]
pub struct PrimalDual<Addr, Uint>(PhantomData<(Addr, Uint)>);

impl<Addr, Uint> MinCostFlow for PrimalDual<Addr, Uint>
where
    Addr: AccountId + Copy + Hash,
    Uint: Amount + CheckedAdd,
{
    type NodeWeight = Addr;
    type EdgeCapacity = Uint;
    type EdgeCost = Uint;
    type GraphIter = BTreeMap<(Node<Addr>, Node<Addr>), Uint>;
    type Error = ();
    type Paths = BTreeMap<(Addr, Addr), Uint>;

    fn min_cost_flow(
        &mut self,
        graph_iter: &Self::GraphIter,
    ) -> Result<(Self::EdgeCapacity, Self::Paths), Self::Error> {
        Ok(mtcs_primal_dual::<Addr, Uint>(graph_iter))
    }
}

pub fn mtcs_primal_dual<Addr, Uint>(
    obligation_list: &BTreeMap<(Node<Addr>, Node<Addr>), Uint>,
) -> (Uint, BTreeMap<(Addr, Addr), Uint>)
where
    Addr: AccountId + Copy + Hash,
    Uint: Amount + CheckedAdd,
{
    let mut graph = DiGraphMap::from_edges(obligation_list.iter().map(|((d, c), capacity)| {
        (
            *d,
            *c,
            EdgeWeight {
                capacity: *capacity,
                cost: Uint::zero(),
            },
        )
    }));

    let mut max_flow = Uint::zero();
    let mut paths = BTreeMap::new();

    loop {
        let balance_source: Uint = graph
            .edges_directed(Node::Source, Direction::Outgoing)
            .map(|(_, _, EdgeWeight { capacity, .. })| *capacity)
            .sum();
        if balance_source == Uint::zero() {
            // finish if there's no remaining flow
            break;
        }

        // find distance vector i.e. distance from source to every other node.
        // This is a potential place for introducing governance, AKA the 'priority-of-claims'.
        // trick: use a high cost for edges whose capacity is exhausted
        let edge_weights = |(_, _, &EdgeWeight { cost, capacity })| {
            if capacity > Uint::zero() {
                cost
            } else {
                Uint::one()
            }
        };
        let distance = dijkstra(&graph, Node::Source, Some(Node::Sink), edge_weights);

        let distance_s_t = distance[&Node::Sink];

        // we define our admissable graph as a subgraph composed of edges that have a
        // `cost <= distance-to-sink` (i.e. `distance_s_t`) and `capacity > 0`
        let admissable_graph = EdgeFiltered::from_fn(&graph, |(_, _, e)| {
            e.cost <= distance_s_t && e.capacity > Uint::zero()
        });

        let path = push_relabel_max_flow(&admissable_graph, Node::Source, Node::Sink).unwrap();
        let path_flow = path
            .iter()
            .filter_map(|((debtor, _), cap)| (debtor == &Node::Source).then_some(*cap))
            .sum();
        if path_flow == Uint::zero() {
            break;
        }

        max_flow += path_flow;

        path.into_iter().for_each(|(edge, amount)| {
            let EdgeWeight { capacity, .. } = &mut graph[edge];
            *capacity -= amount;

            if let (Node::WithId(n1), Node::WithId(n2)) = edge {
                *paths.entry((n1, n2)).or_default() += amount;
            }
        });
    }

    (max_flow, paths)
}
