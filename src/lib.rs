//! # Patricia Merkle Tree
//!
//! This crate implements a Patricia tree. Its implementation is kept as simple
//! as possible to allow for maximum flexibility.

#![deny(warnings)]

pub use self::iter::TreeIterator;
use self::node::{LeafNode, Node};

mod iter;
mod node;
mod util;

/// Patricia Merkle Tree implementation.
///
/// For now, keys are always `[u8; 32]`, which represent `KECCAK256` hashes.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct PatriciaMerkleTree<V> {
    root_node: Option<Node<V>>,
}

impl<V> PatriciaMerkleTree<V> {
    /// Create an empty tree.
    pub fn new() -> Self {
        Self { root_node: None }
    }

    /// Check if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.root_node.is_none()
    }

    /// Retrieves a value given its key.
    pub fn get(&self, key: &[u8; 32]) -> Option<&V> {
        self.root_node
            .as_ref()
            .and_then(|root_node| root_node.get(key, 0))
    }

    /// Insert a key-value into the tree.
    ///
    /// Overwrites and returns the previous value.
    pub fn insert(&mut self, key: &[u8; 32], value: V) -> Option<V> {
        if let Some(root_node) = self.root_node.take() {
            let (root_node, old_value) = root_node.insert(key, value, 0);
            self.root_node = Some(root_node);

            old_value
        } else {
            self.root_node = Some(LeafNode::from_key_value(key.to_owned(), value).into());
            None
        }
    }

    /// Remove a value given its key.
    ///
    /// Returns the removed value.
    pub fn remove(&mut self, key: &[u8; 32]) -> Option<V> {
        if let Some(root_node) = self.root_node.take() {
            let (root_node, old_value) = root_node.remove(key, 0);
            self.root_node = root_node;

            old_value
        } else {
            None
        }
    }

    pub fn iter(&self) -> TreeIterator<V> {
        TreeIterator::new(self)
    }

    pub fn drain_filter<F>(&mut self, mut filter: F) -> Vec<([u8; 32], V)>
    where
        F: FnMut(&[u8; 32], &mut V) -> bool,
    {
        if let Some(root_node) = self.root_node.take() {
            let mut drained_items = Vec::new();
            self.root_node = root_node.drain_filter(&mut filter, &mut drained_items, 0);
            drained_items
        } else {
            Vec::new()
        }
    }
}
