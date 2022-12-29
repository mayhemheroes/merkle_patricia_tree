pub use self::{branch::BranchNode, extension::ExtensionNode, leaf::LeafNode};
use crate::util::KeySegmentIterator;

mod branch;
mod extension;
mod leaf;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Node<V> {
    Branch(BranchNode<V>),
    Extension(ExtensionNode<V>),
    Leaf(LeafNode<V>),
}

impl<V> Node<V> {
    pub fn get(&self, key: &[u8; 32], current_key_offset: usize) -> Option<&V> {
        match self {
            Node::Branch(branch_node) => return branch_node.get(key, current_key_offset),
            Node::Extension(extension_node) => {
                if KeySegmentIterator::new(key)
                    .skip(current_key_offset)
                    .zip(extension_node.prefix().iter().copied())
                    .all(|(a, b)| a == b)
                {
                    return extension_node
                        .child()
                        .get(key, current_key_offset + extension_node.prefix().len());
                }
            }
            Node::Leaf(leaf_node) => {
                if leaf_node.key() == key {
                    return Some(leaf_node.value());
                }
            }
        }

        None
    }

    pub(crate) fn insert(
        self,
        key: &[u8; 32],
        value: V,
        current_key_offset: usize,
    ) -> (Self, Option<V>) {
        match self {
            Node::Branch(branch_node) => {
                let (new_node, old_value) = branch_node.insert(key, value, current_key_offset);
                (new_node.into(), old_value)
            }
            Node::Extension(extension_node) => {
                let (new_node, old_value) = extension_node.insert(key, value, current_key_offset);
                (new_node, old_value)
            }
            Node::Leaf(leaf_node) => {
                let (new_node, old_value) = leaf_node.insert(key, value, current_key_offset);
                (new_node, old_value)
            }
        }
    }

    pub(crate) fn remove(
        self,
        key: &[u8; 32],
        current_key_offset: usize,
    ) -> (Option<Self>, Option<V>) {
        match self {
            Node::Branch(branch_node) => branch_node.remove(key, current_key_offset),
            Node::Extension(extension_node) => extension_node.remove(key, current_key_offset),
            Node::Leaf(leaf_node) => leaf_node.remove(key),
        }
    }

    pub(crate) fn drain_filter<F>(
        self,
        filter: &mut F,
        drained_items: &mut Vec<([u8; 32], V)>,
        current_key_offset: usize,
    ) -> Option<Self>
    where
        F: FnMut(&[u8; 32], &mut V) -> bool,
    {
        match self {
            Node::Branch(branch_node) => {
                branch_node.drain_filter(filter, drained_items, current_key_offset)
            }
            Node::Extension(extension_node) => {
                extension_node.drain_filter(filter, drained_items, current_key_offset)
            }
            Node::Leaf(leaf_node) => leaf_node.drain_filter(filter, drained_items),
        }
    }
}

impl<V> From<BranchNode<V>> for Node<V> {
    fn from(value: BranchNode<V>) -> Self {
        Self::Branch(value)
    }
}

impl<V> From<ExtensionNode<V>> for Node<V> {
    fn from(value: ExtensionNode<V>) -> Self {
        Self::Extension(value)
    }
}

impl<V> From<LeafNode<V>> for Node<V> {
    fn from(value: LeafNode<V>) -> Self {
        Self::Leaf(value)
    }
}
