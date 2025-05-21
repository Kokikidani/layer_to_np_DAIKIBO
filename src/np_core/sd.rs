use std::fmt::Display;

use super::Node;

/// Represents a directed link between two nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SD {
    /// The source node of the directed link.
    pub src: Node,
    /// The destination node of the directed link.
    pub dst: Node,
}

impl Display for SD {
    /// Formats the directed link for display.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.src, self.dst)
    }
}

impl SD {
    /// Creates a new directed link between nodes with the given indices.
    ///
    /// # Arguments
    ///
    /// * `src` - The index of the source node.
    /// * `dst` - The index of the destination node.
    ///
    /// # Returns
    ///
    /// A new `SD` instance representing the directed link.
    pub fn new(src: usize, dst: usize) -> Self {
        Self {
            src: Node::new(src),
            dst: Node::new(dst),
        }
    }

    pub fn new_from_nodes(src: Node, dst: Node) -> Self {
        Self { src, dst }
    }
}

impl From<SD> for (Node, Node) {
    fn from(val: SD) -> Self {
        (val.src, val.dst)
    }
}
impl From<&SD> for (Node, Node) {
    fn from(val: &SD) -> Self {
        (val.src, val.dst)
    }
}