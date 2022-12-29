//! # Patricia Tree
//!
//! This crate implements a Patricia tree. Its implementation is kept as simple
//! as possible to allow for maximum flexibility.

#![deny(warnings)]

pub use self::iter::TreeIterator;
use self::node::{LeafNode, Node};

mod iter;
mod node;
mod util;

/// Patricia Tree implementation.
///
/// For now, keys are always `[u8; 32]`, which represent `KECCAK256` hashes.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct PatriciaTree<V> {
    root_node: Option<Node<V>>,
}

impl<V> PatriciaTree<V> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let tree = PatriciaTree::<i32>::new();
        assert_eq!(tree, pm_tree!());
    }

    #[test]
    fn is_empty() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");

        let tree = pm_tree!(<i32>);
        assert!(tree.is_empty());

        let tree = pm_tree! {
            leaf { key => 42 }
        };
        assert!(!tree.is_empty());
    }

    #[test]
    fn get_some() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let tree = pm_tree! {
            leaf { key => 42 }
        };

        assert_eq!(tree.get(&key), Some(&42));
    }

    #[test]
    fn get_none() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let tree = pm_tree! {
            leaf { key_a => 42 }
        };

        assert_eq!(tree.get(&key_b), None);
    }

    #[test]
    fn insert_empty() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let mut tree = pm_tree!(<i32>);

        let old_value = tree.insert(&key, 42);

        assert_eq!(old_value, None);
        assert_eq!(
            tree,
            pm_tree! {
                leaf { key => 42 }
            },
        );
    }

    #[test]
    fn insert_passthrough() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let mut tree = pm_tree! {
            leaf { key => 42 }
        };

        let old_value = tree.insert(&key, 43);

        assert_eq!(old_value, Some(42));
        assert_eq!(
            tree,
            pm_tree! {
                leaf { key => 43 }
            },
        );
    }

    #[test]
    fn remove_empty() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let mut tree = pm_tree!(<i32>);

        let old_value = tree.remove(&key);

        assert_eq!(old_value, None);
        assert_eq!(tree, pm_tree!());
    }

    #[test]
    fn remove_passthrough() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let mut tree = pm_tree! {
            leaf { key => 42 }
        };

        let old_value = tree.remove(&key);

        assert_eq!(old_value, Some(42));
        assert_eq!(tree, pm_tree!());
    }

    #[test]
    fn iter() {
        let tree = pm_tree!(<i32>);
        assert_eq!(tree.iter(), TreeIterator::new(&tree));
    }

    #[test]
    fn drain_filter_empty() {
        let mut tree = pm_tree!(<i32>);
        assert_eq!(&tree.drain_filter(|_, _| true), &[]);
    }

    #[test]
    fn drain_filter_passthrough() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let mut tree = pm_tree! {
            leaf { key => 42 }
        };

        assert_eq!(&tree.drain_filter(|_, _| true), &[(key, 42)]);
    }
}
