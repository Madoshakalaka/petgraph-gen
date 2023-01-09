//! `petgraph-gen` is a crate that extends [petgraph](https://github.com/petgraph/petgraph)
//! with functions that generate graphs with different properties.

use itertools::Itertools;
use petgraph::graph::IndexType;
use petgraph::{EdgeType, Graph};
use rand::distributions::Bernoulli;
use rand::distributions::Distribution;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashSet;

/// Generates a complete graph with `n` nodes. A complete graph is a graph where
/// each node is connected to every other node. On a directed graph, this means
/// that each node has `n - 1` incoming edges and `n - 1` outgoing edges.
///
/// # Examples
/// ```
/// use petgraph_gen::complete_graph;
/// use petgraph::{Directed, Graph, Undirected};
///
/// let directed_graph: Graph<(), (), Directed> = complete_graph(10);
/// assert_eq!(directed_graph.node_count(), 10);
/// assert_eq!(directed_graph.edge_count(), 90);
///
/// let undirected_graph: Graph<(), (), Undirected> = complete_graph(10);
/// assert_eq!(undirected_graph.node_count(), 10);
/// assert_eq!(undirected_graph.edge_count(), 45);
/// ```
///
pub fn complete_graph<Ty: EdgeType, Ix: IndexType>(n: usize) -> Graph<(), (), Ty, Ix> {
    let mut graph = Graph::with_capacity(n, n * n);
    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(())).collect();
    if n <= 1 {
        return graph;
    }
    if <Ty as EdgeType>::is_directed() {
        nodes.iter().permutations(2).for_each(|edge| {
            graph.add_edge(*edge[0], *edge[1], ());
        });
    } else {
        nodes.iter().combinations(2).for_each(|edge| {
            graph.add_edge(*edge[0], *edge[1], ());
        });
    };
    graph
}

/// Generates an empty graph with `n` nodes and no edges.
///
/// # Examples
/// ```
/// use petgraph::Graph;
/// use petgraph_gen::empty_graph;
///
/// let graph: Graph<(), ()> = empty_graph(5);
/// assert_eq!(graph.node_count(), 5);
/// assert_eq!(graph.edge_count(), 0);
/// ```
pub fn empty_graph<Ty: EdgeType, Ix: IndexType>(n: usize) -> Graph<(), (), Ty, Ix> {
    let mut graph = Graph::with_capacity(n, 0);
    (0..n).for_each(|_| {
        graph.add_node(());
    });
    graph
}

/// Generates a star graph with a single center node connected to `n` other nodes. The resulting
/// graph has `n + 1` nodes and `n` edges.
///
/// # Examples
/// ```
/// use petgraph_gen::star_graph;
/// use petgraph::{Directed, Graph, Undirected};
/// use petgraph::graph::NodeIndex;
/// use petgraph::visit::EdgeRef;
///
/// let graph: Graph<(), ()> = star_graph(10);
/// assert_eq!(graph.node_count(), 11);
/// assert_eq!(graph.edge_count(), 10);
/// let center_node = NodeIndex::new(0);
/// for edge in graph.edges(center_node) {
///    assert_eq!(edge.source(), center_node);
/// }
///
pub fn star_graph<Ty: EdgeType, Ix: IndexType>(n: usize) -> Graph<(), (), Ty, Ix> {
    let mut graph = Graph::with_capacity(n + 1, n);
    let center = graph.add_node(());
    (0..n).for_each(|_| {
        let node = graph.add_node(());
        graph.add_edge(center, node, ());
    });
    graph
}

/// Generates a Erdős-Rényi graph with `n` nodes. Edges are selected with probability `p` from the set
/// of all possible edges.
pub fn erdos_renyi_graph<R: Rng + ?Sized, Ty: EdgeType, Ix: IndexType>(
    rng: &mut R,
    n: usize,
    p: f64,
) -> Graph<(), (), Ty, Ix> {
    let mut graph = Graph::with_capacity(n, ((n * n) as f64 * p) as usize);
    let nodes: Vec<_> = (0..n).map(|_| graph.add_node(())).collect();
    if p <= 0.0 {
        return graph;
    }
    if p >= 1.0 {
        return complete_graph::<Ty, Ix>(n);
    }
    let bernoulli_distribution = Bernoulli::new(p).unwrap();

    if <Ty as EdgeType>::is_directed() {
        nodes.iter().permutations(2).for_each(|edge| {
            if bernoulli_distribution.sample(rng) {
                graph.add_edge(*edge[0], *edge[1], ());
            }
        });
    } else {
        nodes.iter().combinations(2).for_each(|edge| {
            if bernoulli_distribution.sample(rng) {
                graph.add_edge(*edge[0], *edge[1], ());
            }
        });
    };
    graph
}

/// Generates a random graph with `n` nodes using the [Barabási-Albert][ba] model. The graph starts
/// with an empty graph of `m` nodes and adds `n - m` additional nodes. Each new node is connected to `m`
/// existing nodes, where the probability of a node being connected to a given node is proportional
/// to the number of edges that node already has.
///
/// # Panics
/// Panics if `m` equals 0 or is greater than or equal to `n`.
///
/// [ba]: https://en.wikipedia.org/wiki/Barab%C3%A1si%E2%80%93Albert_model
pub fn barabasi_albert_graph<R: Rng + ?Sized, Ty: EdgeType, Ix: IndexType>(
    rng: &mut R,
    n: usize,
    m: usize,
) -> Graph<(), (), Ty, Ix> {
    assert!(m >= 1);
    assert!(m < n);

    let mut graph = Graph::with_capacity(n, (n - m) * m);
    let mut repeated_nodes = (0..m).map(|_| graph.add_node(())).collect_vec();

    for _ in m..n {
        let node = graph.add_node(());

        let mut targets = HashSet::new();
        while targets.len() < m {
            let target = repeated_nodes.choose(rng).unwrap();
            targets.insert(*target);
        }
        for target in targets {
            graph.add_edge(node, target, ());
            repeated_nodes.push(node);
            repeated_nodes.push(target);
        }
    }
    graph
}

#[cfg(test)]
mod tests {
    use petgraph::adj::NodeIndex;
    use super::*;
    use petgraph::graph::{DiGraph, UnGraph};
    use petgraph::visit::EdgeRef;
    use rand::rngs::mock::StepRng;

    trait EdgeIndexTuples<Ty: EdgeType, Ix: IndexType> {
        fn edges_as_tuples(&self) -> Vec<(usize, usize)>;
    }

    impl<Ty: EdgeType, Ix: IndexType> EdgeIndexTuples<Ty, Ix> for Graph<(), (), Ty, Ix> {
        fn edges_as_tuples(&self) -> Vec<(usize, usize)> {
            self.raw_edges()
                .iter()
                .map(|edge| (edge.source().index(), edge.target().index()))
                .collect()
        }
    }

    #[test]
    fn test_empty_graph() {
        let graph: DiGraph<(), ()> = empty_graph(9);
        assert_eq!(graph.node_count(), 9);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_star_graph() {
        let graph: DiGraph<(), ()> = star_graph(9);
        assert_eq!(graph.node_count(), 10);
        assert_eq!(graph.edge_count(), 9);

        let center_node = NodeIndex::new(0);
        assert_eq!(graph.edges(center_node).count(), 9);
        for edge in graph.edges(center_node) {
            assert_eq!(edge.source(), center_node);
        }
    }

    #[test]
    fn test_complete_directed_graph() {
        let graph: DiGraph<(), (), u32> = complete_graph(4);
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 12);
        assert_eq!(
            graph.edges_as_tuples(),
            vec![
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 0),
                (1, 2),
                (1, 3),
                (2, 0),
                (2, 1),
                (2, 3),
                (3, 0),
                (3, 1),
                (3, 2)
            ]
        );
    }

    #[test]
    fn test_complete_undirected_graph() {
        let graph: UnGraph<(), (), u16> = complete_graph(4);
        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 6);
        assert_eq!(
            graph.edges_as_tuples(),
            vec![(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
        );
    }

    #[test]
    fn test_empty_erdos_renyi_graph() {
        let mut rng = rand::thread_rng();
        let graph: UnGraph<(), ()> = erdos_renyi_graph(&mut rng, 10, 0.0);
        assert_eq!(graph.node_count(), 10);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_complete_erdos_renyi_graph() {
        let mut rng = rand::thread_rng();
        let graph: UnGraph<(), ()> = erdos_renyi_graph(&mut rng, 10, 1.0);
        assert_eq!(graph.node_count(), 10);
        assert_eq!(graph.edge_count(), 45);
    }

    #[test]
    fn test_erdos_renyi_graph() {
        let mut rng = StepRng::new(0, u64::MAX / 2 + 1);
        let graph: UnGraph<(), ()> = erdos_renyi_graph(&mut rng, 5, 0.5);
        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 5);
        assert_eq!(
            graph.edges_as_tuples(),
            vec![(0, 1), (0, 3), (1, 2), (1, 4), (2, 4)]
        );
    }

    #[test]
    fn test_barabasi_albert_graph() {
        let mut rng = rand::thread_rng();
        let graph: Graph<(), ()> = barabasi_albert_graph(&mut rng, 100, 3);
        assert_eq!(graph.node_count(), 100);
        assert_eq!(graph.edge_count(), 291);
    }
}
