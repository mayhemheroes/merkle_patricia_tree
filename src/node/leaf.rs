use super::{BranchNode, ExtensionNode, Node};
use crate::util::KeySegmentIterator;
use std::mem::swap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LeafNode<V> {
    key: [u8; 32],
    value: V,
}

impl<V> LeafNode<V> {
    pub(crate) fn from_key_value(key: [u8; 32], value: V) -> Self {
        Self { key, value }
    }

    pub(crate) fn key(&self) -> &[u8; 32] {
        &self.key
    }

    pub(crate) fn value(&self) -> &V {
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
