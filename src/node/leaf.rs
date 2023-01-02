use super::{BranchNode, ExtensionNode, Node};
use crate::util::KeySegmentIterator;
use std::mem::swap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LeafNode<V> {
    key: [u8; 32],
    value: V,
}

impl<V> LeafNode<V> {
    pub(crate) const fn from_key_value(key: [u8; 32], value: V) -> Self {
        Self { key, value }
    }

    pub(crate) const fn key(&self) -> &[u8; 32] {
        &self.key
    }

    pub(crate) const fn value(&self) -> &V {
        &self.value
    }

    pub(crate) fn insert(
        mut self,
        key: &[u8; 32],
        value: V,
        current_key_offset: usize,
    ) -> (Node<V>, Option<V>) {
        match KeySegmentIterator::new(key)
            .zip(KeySegmentIterator::new(&self.key))
            .skip(current_key_offset)
            .enumerate()
            .find_map(|(idx, (a, b))| (a != b).then_some((idx, a, b)))
        {
            Some((prefix_len, value_b, value_a)) => {
                let leaf_a = self;
                let leaf_b = LeafNode {
                    key: key.to_owned(),
                    value,
                };

                let branch_node = BranchNode::from_choices({
                    let mut choices: [Option<Box<Node<V>>>; 16] = Default::default();

                    choices[value_a as usize] = Some(Box::new(leaf_a.into()));
                    choices[value_b as usize] = Some(Box::new(leaf_b.into()));

                    choices
                });

                let node: Node<V> = if prefix_len == 0 {
                    branch_node.into()
                } else {
                    ExtensionNode::from_prefix_child(
                        KeySegmentIterator::new(key).take(prefix_len).collect(),
                        branch_node,
                    )
                    .into()
                };

                (node, None)
            }
            _ => {
                let mut value = value;
                swap(&mut value, &mut self.value);
                (self.into(), Some(value))
            }
        }
    }

    pub(crate) fn remove(self, key: &[u8; 32]) -> (Option<Node<V>>, Option<V>) {
        if *key == self.key {
            (None, Some(self.value))
        } else {
            (Some(self.into()), None)
        }
    }

    pub(crate) fn drain_filter<F>(
        mut self,
        filter: &mut F,
        drained_items: &mut Vec<([u8; 32], V)>,
    ) -> Option<Node<V>>
    where
        F: FnMut(&[u8; 32], &mut V) -> bool,
    {
        if filter(&self.key, &mut self.value) {
            drained_items.push((self.key, self.value));
            None
        } else {
            Some(self.into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{pm_tree_branch, pm_tree_key};

    #[test]
    fn from_key_value() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node = LeafNode::from_key_value(key, 42i32);

        assert_eq!(node.key, key);
        assert_eq!(node.value, 42);
    }

    #[test]
    fn key() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node = pm_tree_branch! {
            leaf { key => 42i32 }
        };

        assert_eq!(node.key(), &key);
    }

    #[test]
    fn value() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node = pm_tree_branch! {
            leaf { key => 42i32 }
        };

        assert_eq!(node.value(), &42);
    }

    #[test]
    fn insert_overwrite() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node = pm_tree_branch! {
            leaf { key => 42i32 }
        };

        let (node, old_value) = node.insert(&key, 43, 0);

        assert_eq!(old_value, Some(42));
        assert!(matches!(node, Node::Leaf(_)));
        match node {
            Node::Leaf(node) => {
                assert_eq!(
                    node.key,
                    pm_tree_key!(
                        "0000000000000000000000000000000000000000000000000000000000000000"
                    ),
                );
                assert_eq!(node.value, 43);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn insert_branch() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            leaf { key_a => 42i32 }
        };

        let (node, old_value) = node.insert(&key_b, 43, 0);

        assert_eq!(old_value, None);
        assert_eq!(
            node,
            pm_tree_branch! {
                branch {
                    0 => leaf { key_a => 42 },
                    1 => leaf { key_b => 43 },
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
            pm_tree_key!("0100000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            leaf { key_a => 42i32 }
        };

        let (node, old_value) = node.insert(&key_b, 43, 0);

        assert_eq!(old_value, None);
        assert_eq!(
            node,
            pm_tree_branch! {
                extension { "0", branch {
                    0 => leaf { key_a => 42 },
                    1 => leaf { key_b => 43 },
                } }
            }
            .into(),
        );
    }

    #[test]
    fn remove_self() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node = pm_tree_branch! {
            leaf { key => 42i32 }
        };

        let (node, old_value) = node.remove(&key);

        assert_eq!(node, None);
        assert_eq!(old_value, Some(42));
    }

    #[test]
    fn remove_ignore() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            leaf { key_a => 42i32 }
        };

        let (node, old_value) = node.remove(&key_b);

        assert_eq!(
            node,
            Some(
                pm_tree_branch! {
                    leaf { key_a => 42 }
                }
                .into()
            )
        );
        assert_eq!(old_value, None);
    }

    #[test]
    fn drain_filter_self() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node = pm_tree_branch! {
            leaf { key => 42i32 }
        };

        let mut drained_items = Vec::new();
        let node = node.drain_filter(&mut |_, _| true, &mut drained_items);

        assert_eq!(node, None);
        assert_eq!(&drained_items, &[(key, 42)]);
    }

    #[test]
    fn drain_filter_ignore() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let node = pm_tree_branch! {
            leaf { key => 42i32 }
        };

        let mut drained_items = Vec::new();
        let node = node.drain_filter(&mut |_, _| false, &mut drained_items);

        assert_eq!(
            node,
            Some(
                pm_tree_branch! {
                    leaf { key => 42 }
                }
                .into()
            )
        );
        assert_eq!(&drained_items, &[]);
    }
}
