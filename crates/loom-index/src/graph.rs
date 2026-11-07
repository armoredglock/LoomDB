use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

/// Compressed Sparse Row (CSR) Graph Representation.
/// High-throughput zero-copy adjacency traversal with cache-friendly contiguous arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSRGraph {
    pub num_nodes: usize,
    /// Offset array: node `i`'s neighbors are at indices `node_offsets[i] .. node_offsets[i+1]`
    pub node_offsets: Vec<u32>,
    /// Contiguous array of destination node IDs
    pub edge_targets: Vec<u32>,
    /// Contiguous array of edge weights
    pub edge_weights: Vec<f32>,
}

impl CSRGraph {
    pub fn new() -> Self {
        Self {
            num_nodes: 0,
            node_offsets: vec![0],
            edge_targets: Vec::new(),
            edge_weights: Vec::new(),
        }
    }

    /// Construct CSR graph from an edge list: (from_node, to_node, weight).
    pub fn from_edge_list(num_nodes: usize, mut edges: Vec<(u32, u32, f32)>) -> Self {
        // Sort edges by source node, then destination
        edges.sort_by(|a, b| {
            if a.0 != b.0 {
                a.0.cmp(&b.0)
            } else {
                a.1.cmp(&b.1)
            }
        });

        let mut node_offsets = Vec::with_capacity(num_nodes + 1);
        let mut edge_targets = Vec::with_capacity(edges.len());
        let mut edge_weights = Vec::with_capacity(edges.len());

        let mut current_node = 0u32;
        node_offsets.push(0);

        for (from, to, weight) in edges {
            while current_node < from {
                current_node += 1;
                node_offsets.push(edge_targets.len() as u32);
            }
            edge_targets.push(to);
            edge_weights.push(weight);
        }

        while (node_offsets.len() as usize) <= num_nodes {
            node_offsets.push(edge_targets.len() as u32);
        }

        Self {
            num_nodes,
            node_offsets,
            edge_targets,
            edge_weights,
        }
    }

    /// Retrieve zero-copy contiguous slice of adjacent neighbors for a given node.
    pub fn neighbors(&self, node: u32) -> &[u32] {
        let n = node as usize;
        if n >= self.num_nodes || n + 1 >= self.node_offsets.len() {
            return &[];
        }
        let start = self.node_offsets[n] as usize;
        let end = self.node_offsets[n + 1] as usize;
        &self.edge_targets[start..end]
    }

    /// Retrieve zero-copy contiguous slice of edge weights for a given node.
    pub fn weights(&self, node: u32) -> &[f32] {
        let n = node as usize;
        if n >= self.num_nodes || n + 1 >= self.node_offsets.len() {
            return &[];
        }
        let start = self.node_offsets[n] as usize;
        let end = self.node_offsets[n + 1] as usize;
        &self.edge_weights[start..end]
    }

    /// Out-degree (number of outgoing edges) for node.
    pub fn degree(&self, node: u32) -> usize {
        let n = node as usize;
        if n >= self.num_nodes || n + 1 >= self.node_offsets.len() {
            0
        } else {
            (self.node_offsets[n + 1] - self.node_offsets[n]) as usize
        }
    }

    /// Breadth-First Search (BFS) returning all reachable nodes within `max_depth` hops.
    pub fn bfs_reachability(&self, start_node: u32, max_depth: usize) -> Vec<u32> {
        let mut visited = vec![false; self.num_nodes];
        let mut queue = VecDeque::new();
        let mut reachable = Vec::new();

        if (start_node as usize) >= self.num_nodes {
            return reachable;
        }

        visited[start_node as usize] = true;
        queue.push_back((start_node, 0usize));

        while let Some((node, depth)) = queue.pop_front() {
            reachable.push(node);
            if depth < max_depth {
                for &neighbor in self.neighbors(node) {
                    let n_idx = neighbor as usize;
                    if n_idx < self.num_nodes && !visited[n_idx] {
                        visited[n_idx] = true;
                        queue.push_back((neighbor, depth + 1));
                    }
                }
            }
        }

        reachable
    }

    /// Compute shortest path using Dijkstra's algorithm.
    pub fn shortest_path(&self, start: u32, target: u32) -> Option<(f32, Vec<u32>)> {
        if (start as usize) >= self.num_nodes || (target as usize) >= self.num_nodes {
            return None;
        }

        #[derive(Copy, Clone, PartialEq)]
        struct State {
            cost: f32,
            node: u32,
        }

        impl Eq for State {}
        impl Ord for State {
            fn cmp(&self, other: &Self) -> Ordering {
                other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
            }
        }
        impl PartialOrd for State {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut dist = vec![f32::INFINITY; self.num_nodes];
        let mut prev = vec![None; self.num_nodes];
        let mut heap = BinaryHeap::new();

        dist[start as usize] = 0.0;
        heap.push(State { cost: 0.0, node: start });

        while let Some(State { cost, node }) = heap.pop() {
            if node == target {
                // Reconstruct path
                let mut path = Vec::new();
                let mut curr = Some(target);
                while let Some(n) = curr {
                    path.push(n);
                    curr = prev[n as usize];
                }
                path.reverse();
                return Some((cost, path));
            }

            if cost > dist[node as usize] {
                continue;
            }

            let neighbors = self.neighbors(node);
            let weights = self.weights(node);

            for (&next, &weight) in neighbors.iter().zip(weights.iter()) {
                let next_cost = cost + weight;
                if next_cost < dist[next as usize] {
                    dist[next as usize] = next_cost;
                    prev[next as usize] = Some(node);
                    heap.push(State { cost: next_cost, node: next });
                }
            }
        }

        None
    }
}

impl Default for CSRGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csr_graph_construction_and_traversal() {
        // Construct graph: 0 -> 1 (w=2.0), 0 -> 2 (w=4.0), 1 -> 2 (w=1.0), 2 -> 3 (w=3.0)
        let edges = vec![
            (0, 1, 2.0),
            (0, 2, 4.0),
            (1, 2, 1.0),
            (2, 3, 3.0),
        ];

        let graph = CSRGraph::from_edge_list(4, edges);

        assert_eq!(graph.degree(0), 2);
        assert_eq!(graph.neighbors(0), &[1, 2]);
        assert_eq!(graph.weights(0), &[2.0, 4.0]);

        assert_eq!(graph.degree(1), 1);
        assert_eq!(graph.neighbors(1), &[2]);

        // BFS from node 0 with depth 1
        let depth1 = graph.bfs_reachability(0, 1);
        assert_eq!(depth1, vec![0, 1, 2]);

        // Dijkstra shortest path: 0 -> 1 -> 2 -> 3 (cost = 2.0 + 1.0 + 3.0 = 6.0)
        let (cost, path) = graph.shortest_path(0, 3).unwrap();
        assert_eq!(cost, 6.0);
        assert_eq!(path, vec![0, 1, 2, 3]);
    }
}
