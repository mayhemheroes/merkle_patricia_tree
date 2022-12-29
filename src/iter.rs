use crate::{
    node::{BranchNode, Node},
    PatriciaMerkleTree,
};

/// Iterator state (for each node, like a stack).
///
/// The `Node<V>` enum can't be used because it doesn't handle special cases such as the
/// `ExtensionNode`'s child well.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct NodeState<'a, V> {
    node: &'a BranchNode<V>,
    state: usize,
}

pub struct TreeIterator<'a, V> {
    tree: Option<&'a PatriciaMerkleTree<V>>,
    state: Vec<NodeState<'a, V>>,
}

impl<'a, V> TreeIterator<'a, V> {
    pub(crate) fn new(tree: &'a PatriciaMerkleTree<V>) -> Self {
        Self {
            tree: Some(tree),
            state: vec![],
        }
    }
}

impl<'a, V> Iterator for TreeIterator<'a, V> {
    type Item = (&'a [u8; 32], &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(tree) = self.tree.take() {
                self.state.push(match &tree.root_node {
                    Some(root_node) => {
                        let current_node = root_node;
                        NodeState {
                            node: match current_node {
                                Node::Branch(branch_node) => branch_node,
                                Node::Extension(extension_node) => extension_node.child(),
                                Node::Leaf(leaf_node) => {
                                    break Some((leaf_node.key(), leaf_node.value()))
                                }
                            },
                            state: 0,
                        }
                    }
                    None => break None,
                });
            }

            match self.state.pop() {
                Some(last_state) if last_state.state < last_state.node.choices().len() => {
                    self.state.push(NodeState {
                        node: last_state.node,
                        state: last_state.state + 1,
                    });

                    if let Some(choice) = &last_state.node.choices()[last_state.state] {
                        self.state.push(NodeState {
                            node: match choice.as_ref() {
                                Node::Branch(branch_node) => branch_node,
                                Node::Extension(extension_node) => extension_node.child(),
                                Node::Leaf(leaf_node) => {
                                    break Some((leaf_node.key(), leaf_node.value()))
                                }
                            },
                            state: 0,
                        });
                    }
                }
                None => break None,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{pm_tree, pm_tree_key};

    #[test]
    fn iterate_empty() {
        let tree = pm_tree!(<()>);
        assert_eq!(&tree.iter().collect::<Vec<_>>(), &[]);
    }

    #[test]
    fn iterate_branches() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");
        let key_c =
            pm_tree_key!("8000000000000000000000000000000000000000000000000000000000000000");
        let key_d =
            pm_tree_key!("f000000000000000000000000000000000000000000000000000000000000000");

        let tree = pm_tree! {
            branch {
                0x00 => leaf { key_a => 1 },
                0x01 => leaf { key_b => 2 },
                0x08 => leaf { key_c => 3 },
                0x0f => leaf { key_d => 4 },
            }
        };

        assert_eq!(
            &tree.iter().collect::<Vec<_>>(),
            &[(&key_a, &1), (&key_b, &2), (&key_c, &3), (&key_d, &4)],
        );
    }

    #[test]
    fn iterate_extension() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("0001000000000000000000000000000000000000000000000000000000000000");

        let pm_tree = pm_tree! {
            extension { "000", branch {
                0 => leaf { key_a => 0 },
                1 => leaf { key_b => 1 },
            } }
        };

        assert_eq!(
            &pm_tree.iter().collect::<Vec<_>>(),
            &[(&key_a, &0), (&key_b, &1)],
        );
    }

    #[test]
    fn iterate_leaf() {
        let key = pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let pm_tree = pm_tree! {
            leaf { key => 42 }
        };

        assert_eq!(&pm_tree.iter().collect::<Vec<_>>(), &[(&key, &42)]);
    }
}
