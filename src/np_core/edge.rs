use std::fmt::Display;

use super::Node;

/// Represents an undirected edge between two nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Edge {
    /// The first node connected by the edge.
    pub src: Node,
    /// The second node connected by the edge.
    pub dst: Node,
}

impl Display for Edge {
    /// Formats the edge for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.src, self.dst)
    }
}

impl Edge {
    /// Creates a new undirected edge between nodes with the given indices.
    ///
    /// # Arguments
    ///
    /// * `src` - The index of the first node.
    /// * `dst` - The index of the second node.
    ///
    /// # Returns
    ///
    /// A new `Edge` instance representing the undirected edge.
    pub fn new(src: usize, dst: usize) -> Self {
        Self {
            src: Node::new(src),
            dst: Node::new(dst),
        }
    }
}
