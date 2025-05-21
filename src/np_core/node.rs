use std::fmt::Display;

use serde_derive::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Deserialize)]
pub struct Node {
    pub(super) value: usize,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:2}", self.value)
    }
}

impl From<Node> for usize {
    fn from(val: Node) -> Self {
        val.value
    }
}

impl Node {
    pub fn new(value: usize) -> Self {
        Self { value }
    }
}
