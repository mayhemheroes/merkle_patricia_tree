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
            Node::Branch(branch_node) => match branch_node.remove(key, current_key_offset) {
                (Some((_, x)), y) => (Some(x), y),
                (None, x) => (None, x),
            },
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
            Node::Branch(branch_node) => branch_node
                .drain_filter(filter, drained_items, current_key_offset)
                .map(|(_, x)| x),
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{pm_tree_branch, pm_tree_key};

    #[test]
    fn get_branch() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node: Node<_> = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        }
        .into();

        assert_eq!(node.get(&key_a, 0), Some(&42));
        assert_eq!(node.get(&key_b, 0), Some(&43));
    }

    #[test]
    fn get_extension() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("0001000000000000000000000000000000000000000000000000000000000000");

        let node: Node<_> = pm_tree_branch! {
            extension { "000", branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            } }
        }
        .into();

        assert_eq!(node.get(&key_a, 0), Some(&42));
        assert_eq!(node.get(&key_b, 0), Some(&43));
    }

    #[test]
    fn get_leaf() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node: Node<_> = pm_tree_branch! {
            leaf { key => 42 }
        }
        .into();

        assert_eq!(node.get(&key, 0), Some(&42));
    }

    #[test]
    fn get_none() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("0001000000000000000000000000000000000000000000000000000000000000");

        let node: Node<_> = pm_tree_branch! {
            leaf { key_a => 42 }
        }
        .into();

        assert_eq!(node.get(&key_b, 0), None);
    }

    #[test]
    fn insert_branch() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node: Node<_> = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        }
        .into();

        let (node, old_value) = node.insert(&key_b, 44, 0);

        assert_eq!(old_value, Some(43));
        assert_eq!(
            node,
            pm_tree_branch! {
                branch {
                    0 => leaf { key_a => 42 },
                    1 => leaf { key_b => 44 },
                }
            }
            .into(),
        );
    }

    #[test]
    fn insert_extension() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("0001000000000000000000000000000000000000000000000000000000000000");

        let node: Node<_> = pm_tree_branch! {
            extension { "000", branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            } }
        }
        .into();

        let (node, old_value) = node.insert(&key_b, 44, 0);

        assert_eq!(old_value, Some(43));
        assert_eq!(
            node,
            pm_tree_branch! {
                extension { "000", branch {
                    0 => leaf { key_a => 42 },
                    1 => leaf { key_b => 44 },
                } }
            }
            .into(),
        );
    }

    #[test]
    fn insert_leaf() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node: Node<_> = pm_tree_branch! {
            leaf { key => 42 }
        }
        .into();

        let (node, old_value) = node.insert(&key, 43, 0);

        assert_eq!(old_value, Some(42));
        assert_eq!(
            node,
            pm_tree_branch! {
                leaf { key => 43 }
            }
            .into(),
        );
    }

    #[test]
    fn remove_branch() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node: Node<_> = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        }
        .into();

        let (node, old_value) = node.remove(&key_b, 0);

        assert_eq!(old_value, Some(43));
        assert_eq!(
            node,
            Some(
                pm_tree_branch! {
                    leaf { key_a => 42 }
                }
                .into()
            ),
        );
    }

    #[test]
    fn remove_branch_all() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node: Node<_> = pm_tree_branch! {
            // Note: This construct is invalid and can't be obtained using the public API: branches
            //   always have more than one branch, otherwise they get removed.
            branch {
                0 => leaf { key => 42 },
            }
        }
        .into();

        let (node, old_value) = node.remove(&key, 0);

        assert_eq!(old_value, Some(42));
        assert_eq!(node, None);
    }

    #[test]
    fn remove_extension() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("0001000000000000000000000000000000000000000000000000000000000000");

        let node: Node<_> = pm_tree_branch! {
            extension { "000", branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            } }
        }
        .into();

        let (node, old_value) = node.remove(&key_b, 0);

        assert_eq!(old_value, Some(43));
        assert_eq!(
            node,
            Some(
                pm_tree_branch! {
                    leaf { key_a => 42 }
                }
                .into()
            ),
        );
    }

    #[test]
    fn remove_leaf() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node: Node<_> = pm_tree_branch! {
            leaf { key => 42 }
        }
        .into();

        let (node, old_value) = node.remove(&key, 0);

        assert_eq!(old_value, Some(42));
        assert_eq!(node, None);
    }

    #[test]
    fn drain_filter_branch() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node: Node<_> = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        }
        .into();

        let mut drained_items = Vec::new();
        let node = node.drain_filter(&mut |_, _| true, &mut drained_items, 0);

        assert_eq!(node, None);
        assert_eq!(&drained_items, &[(key_a, 42), (key_b, 43)]);
    }

    #[test]
    fn drain_filter_extension() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("0001000000000000000000000000000000000000000000000000000000000000");

        let node: Node<_> = pm_tree_branch! {
            extension { "000", branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            } }
        }
        .into();

        let mut drained_items = Vec::new();
        let node = node.drain_filter(&mut |_, _| true, &mut drained_items, 0);

        assert_eq!(node, None);
        assert_eq!(&drained_items, &[(key_a, 42), (key_b, 43)]);
    }

    #[test]
    fn drain_filter_leaf() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node: Node<_> = pm_tree_branch! {
            leaf { key => 42 }
        }
        .into();

        let mut drained_items = Vec::new();
        let node = node.drain_filter(&mut |_, _| true, &mut drained_items, 0);

        assert_eq!(node, None);
        assert_eq!(&drained_items, &[(key, 42)]);
    }

    #[test]
    fn from_branch_node() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let specific_node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        };

        let node: Node<_> = specific_node.clone().into();
        assert_eq!(node, Node::Branch(specific_node));
    }

    #[test]
    fn from_extension_node() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("0001000000000000000000000000000000000000000000000000000000000000");

        let specific_node = pm_tree_branch! {
            extension { "000", branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            } }
        };

        let node: Node<_> = specific_node.clone().into();
        assert_eq!(node, Node::Extension(specific_node));
    }

    #[test]
    fn from_leaf_node() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let specific_node = pm_tree_branch! {
            leaf { key => 42 }
        };

        let node: Node<_> = specific_node.clone().into();
        assert_eq!(node, Node::Leaf(specific_node));
    }
}
