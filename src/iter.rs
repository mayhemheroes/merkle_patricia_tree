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
