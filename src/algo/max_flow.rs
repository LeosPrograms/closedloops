use alloc::collections::{btree_map::Entry::Vacant, BTreeMap, VecDeque};
use core::cmp::{max, min};
use core::hash::Hash;

use num_traits::{CheckedAdd, Zero};
use petgraph::graph::NodeIndex;
use petgraph::visit::{EdgeRef, IntoEdgeReferences, IntoNodeIdentifiers};
use petgraph::Graph;

use crate::algo::mcmf::primal_dual::EdgeWeight;
use crate::amount::Amount;

pub type NodePair<NodeId> = (NodeId, NodeId);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MaxFlowError {
    ArithmeticOverflow,
}

struct Node<N, Int> {
    orig_id: N,
    excess: Int,
    label: usize,
}

impl<N, Int: Zero> Node<N, Int> {
    fn new(orig_id: N) -> Node<N, Int> {
        Node {
            orig_id,
            excess: Int::zero(),
            label: 0,
        }
    }
}

struct Edge<Int> {
    capacity: Int,
    flow: Int,
}

impl<Int: Zero> Edge<Int> {
    fn new(capacity: Int) -> Edge<Int> {
        Edge {
            capacity,
            flow: Int::zero(),
        }
    }
}

struct State<N, Int> {
    graph: Graph<Node<N, Int>, ()>,
    // We need random access to the edges, so it is faster to store them in a
    // hashmap rather than in `graph`.
    edges: BTreeMap<(NodeId, NodeId), Edge<Int>>,
    target: NodeId,
    active_queue: VecDeque<NodeId>,
}

type PRGraph<N, Int> = Graph<Node<N, Int>, ()>;
type NodeId = NodeIndex<u32>;

impl<N: Copy + Ord, Int: Amount + CheckedAdd> State<N, Int> {
    fn push(&mut self, u: NodeId, v: NodeId) -> Result<(), MaxFlowError> {
        let new_flow = {
            let u_data = self.graph.node_weight(u).unwrap();
            let v_data = self.graph.node_weight(v).unwrap();
            let e_data = self.edges.get(&(u, v)).unwrap();

            debug_assert!(u_data.excess > Int::zero());
            debug_assert!(u_data.label == v_data.label + 1);

            min(u_data.excess, e_data.capacity - e_data.flow)
        };
        self.add_excess(u, -new_flow)?;
        self.add_excess(v, new_flow)?;
        self.edges.get_mut(&(u, v)).unwrap().flow += new_flow;
        self.edges.get_mut(&(v, u)).unwrap().flow -= new_flow;
        Ok(())
    }

    fn has_capacity(&self, u: NodeId, v: NodeId) -> bool {
        let e = self.edges.get(&(u, v)).unwrap();
        e.capacity > e.flow
    }

    fn can_push(&self, u: NodeId, v: NodeId) -> bool {
        self.has_capacity(u, v) && self.graph[u].label == self.graph[v].label + 1
    }

    fn add_excess(&mut self, u: NodeId, amount: Int) -> Result<(), MaxFlowError> {
        debug_assert!(amount != Int::zero());

        // The target node never has any excess inflow, since it can just gobble it up.
        if u == self.target {
            return Ok(());
        }

        let node = self.graph.node_weight_mut(u).unwrap();
        // We should never try to push more flow than the node has available.
        // There is one special case: the start node always has non-positive
        // excess flow.
        debug_assert!(node.excess <= Int::zero() || node.excess >= -amount);
        if node.excess == Int::zero() {
            // We weren't active before, but we are now.
            self.active_queue.push_back(u);
        }
        node.excess = node
            .excess
            .checked_add(&amount)
            .ok_or(MaxFlowError::ArithmeticOverflow)?;
        Ok(())
    }

    // Keep pushing excess flow to neighbors until we can't any more.
    fn discharge(&mut self, u: NodeId) -> Result<(), MaxFlowError> {
        let mut nbrs = self.graph.neighbors(u).detach();
        while self.graph[u].excess > Int::zero() {
            if let Some(v) = nbrs.next_node(&self.graph) {
                if self.can_push(u, v) {
                    self.push(u, v)?;
                }
            } else {
                self.relabel(u);
                nbrs = self.graph.neighbors(u).detach();
            }
        }
        Ok(())
    }

    fn relabel(&mut self, u: NodeId) {
        let min_nbr_label = self
            .graph
            .neighbors(u)
            .filter(|v| self.has_capacity(u, *v))
            .map(|v| self.graph.node_weight(v).unwrap().label)
            .min()
            .expect("bug: tried to relabel a node with no outgoing edges");
        self.graph.node_weight_mut(u).unwrap().label = min_nbr_label + 1;
    }

    fn new<G>(g: G, source: G::NodeId, target: G::NodeId) -> State<G::NodeId, Int>
    where
        G: IntoEdgeReferences<EdgeWeight = EdgeWeight<Int, Int>, NodeId = N> + IntoNodeIdentifiers,
        G::NodeId: Hash + Eq,
    {
        // Map from nodes of `g` to nodes in `pr_graph`.
        let mut node_map = BTreeMap::new();
        let mut pr_graph = PRGraph::new();
        let mut edges = BTreeMap::new();

        for n in g.node_identifiers() {
            let pr_id = pr_graph.add_node(Node::new(n));
            node_map.insert(n, pr_id);
        }
        for e in g.edge_references() {
            let u = node_map[&e.source()];
            let v = node_map[&e.target()];
            pr_graph.add_edge(u, v, ());
            edges.insert((u, v), Edge::new(max(e.weight().capacity, Int::zero())));
        }

        // The algorithm requires that every edge has its reversal present.
        for e in g.edge_references() {
            let u = node_map[&e.source()];
            let v = node_map[&e.target()];
            if let Vacant(e) = edges.entry((v, u)) {
                e.insert(Edge::new(Int::zero()));
                pr_graph.add_edge(v, u, ());
            }
        }

        let pr_source = *node_map
            .get(&source)
            .expect("source node isn't in the graph");
        pr_graph[pr_source].label = pr_graph.node_count();
        let mut nbrs = pr_graph.neighbors(pr_source).detach();
        let mut active = VecDeque::new();

        while let Some(v) = nbrs.next_node(&pr_graph) {
            let cap = edges[&(pr_source, v)].capacity;
            edges.get_mut(&(pr_source, v)).unwrap().flow = cap;
            edges.get_mut(&(v, pr_source)).unwrap().flow = -cap;
            pr_graph[v].excess += cap;
            pr_graph[pr_source].excess -= cap;
            active.push_back(v);
        }

        State {
            edges,
            graph: pr_graph,
            target: *node_map
                .get(&target)
                .expect("target node isn't in the graph"),
            active_queue: active,
        }
    }

    fn run(&mut self) -> Result<(), MaxFlowError> {
        while let Some(u) = self.active_queue.pop_front() {
            self.discharge(u)?;
        }
        Ok(())
    }
}

/// Computes a max flow from `source` to `target` in the weighted graph `g` using the push-relabel
/// algorithm.
///
/// The edge weights in `g` are interpreted as edge capacities -- negative weights are treated the
/// same as zero weights.
///
/// Returns `BTreeMap` that maps ordered pairs of vertices to the flow between them. The map only
/// contains pairs of vertices with a strictly positive flow. Returns an error if an arithmetic
/// overflow occurred.
///
/// Panics if `source` or `target` is an invalid node index for `g`.
pub fn push_relabel_max_flow<G, Int>(
    g: G,
    source: G::NodeId,
    target: G::NodeId,
) -> Result<BTreeMap<NodePair<G::NodeId>, Int>, MaxFlowError>
where
    G: IntoEdgeReferences<EdgeWeight = EdgeWeight<Int, Int>> + IntoNodeIdentifiers,
    G::NodeId: Clone + Hash + Eq + Ord,
    Int: Amount + CheckedAdd,
{
    let mut state = State::new(g, source, target);
    state.run()?;

    let graph = state.graph;
    let flow = state
        .edges
        .into_iter()
        .filter(|(_, data)| data.flow > Int::zero())
        .map(|((u, v), data)| ((graph[u].orig_id, graph[v].orig_id), data.flow))
        .collect::<BTreeMap<_, _>>();

    Ok(flow)
}
