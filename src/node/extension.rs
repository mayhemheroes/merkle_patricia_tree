use super::{BranchNode, LeafNode, Node};
use crate::util::KeySegmentIterator;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ExtensionNode<V> {
    // Each value is a nibble.
    prefix: Vec<u8>,
    // The only child type that makes sense here is a branch node, therefore there's no need to wrap
    // it in a `Node<V>`.
    child: BranchNode<V>,
}

impl<V> ExtensionNode<V> {
    pub(crate) fn from_prefix_child(prefix: Vec<u8>, child: BranchNode<V>) -> Self {
        Self { prefix, child }
    }

    pub(crate) fn prefix(&self) -> &[u8] {
        &self.prefix
    }

    pub(crate) const fn child(&self) -> &BranchNode<V> {
        &self.child
    }

    pub(crate) fn insert(
        mut self,
        key: &[u8; 32],
        value: V,
        current_key_offset: usize,
    ) -> (Node<V>, Option<V>) {
        match KeySegmentIterator::new(key)
            .skip(current_key_offset)
            .enumerate()
            .zip(self.prefix.iter().copied())
            .find_map(|((idx, a), b)| (a != b).then_some((idx, a, b)))
        {
            Some((prefix_len, value_b, value_a)) => (
                if prefix_len == 0 {
                    self.prefix.remove(0);

                    BranchNode::from_choices({
                        let mut choices: [Option<Box<Node<V>>>; 16] = Default::default();

                        choices[value_a as usize] = Some(Box::new(self.into()));
                        choices[value_b as usize] = Some(Box::new(
                            LeafNode::from_key_value(key.to_owned(), value).into(),
                        ));

                        choices
                    })
                    .into()
                } else {
                    let branch_node = BranchNode::from_choices({
                        let mut choices: [Option<Box<Node<V>>>; 16] = Default::default();

                        choices[value_a as usize] =
                            Some(Box::new(if prefix_len != self.prefix.len() - 1 {
                                let (_, new_prefix) = self.prefix.split_at(prefix_len + 1);
                                ExtensionNode {
                                    prefix: new_prefix.to_vec(),
                                    child: self.child,
                                }
                                .into()
                            } else {
                                self.child.into()
                            }));
                        choices[value_b as usize] = Some(Box::new(
                            LeafNode::from_key_value(key.to_owned(), value).into(),
                        ));

                        choices
                    });

                    self.prefix.truncate(prefix_len);
                    ExtensionNode {
                        prefix: self.prefix,
                        child: branch_node,
                    }
                    .into()
                },
                None,
            ),
            None => {
                let old_value;
                (self.child, old_value) =
                    self.child
                        .insert(key, value, current_key_offset + self.prefix.len());
                (self.into(), old_value)
            }
        }
    }

    pub(crate) fn remove(
        mut self,
        key: &[u8; 32],
        current_key_offset: usize,
    ) -> (Option<Node<V>>, Option<V>) {
        let (new_child, old_value) = self
            .child
            .remove(key, current_key_offset + self.prefix.len());

        (
            match new_child {
                Some((_, Node::Branch(branch_node))) => {
                    self.child = branch_node;
                    Some(self.into())
                }
                Some((index, Node::Extension(extension_node))) => {
                    self.prefix.push(match index {
                        Some(x) => x,
                        None => unreachable!(),
                    });
                    self.prefix.extend(extension_node.prefix.into_iter());
                    self.child = extension_node.child;
                    Some(self.into())
                }
                Some((_, Node::Leaf(leaf_node))) => Some(leaf_node.into()),
                None => None,
            },
            old_value,
        )
    }

    pub(crate) fn drain_filter<F>(
        mut self,
        filter: &mut F,
        drained_items: &mut Vec<([u8; 32], V)>,
        current_key_offset: usize,
    ) -> Option<Node<V>>
    where
        F: FnMut(&[u8; 32], &mut V) -> bool,
    {
        let new_child = self.child.drain_filter(
            filter,
            drained_items,
            current_key_offset + self.prefix.len(),
        );

        match new_child {
            Some(Node::Branch(branch_node)) => {
                self.child = branch_node;
                Some(self.into())
            }
            Some(Node::Extension(extension_node)) => {
                self.prefix.extend(extension_node.prefix.into_iter());
                self.child = extension_node.child;
                Some(self.into())
            }
            Some(Node::Leaf(leaf_node)) => Some(leaf_node.into()),
            None => None,
        }
    }
}
