use super::{LeafNode, Node};
use crate::util::KeySegmentIterator;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BranchNode<V> {
    choices: [Option<Box<Node<V>>>; 16],
}

impl<V> BranchNode<V> {
    pub(crate) fn from_choices(choices: [Option<Box<Node<V>>>; 16]) -> Self {
        Self { choices }
    }

    pub(crate) fn choices(&self) -> &[Option<Box<Node<V>>>; 16] {
        &self.choices
    }

    pub(crate) fn get(&self, key: &[u8; 32], current_key_offset: usize) -> Option<&V> {
        self.choices[KeySegmentIterator::nth(key, current_key_offset) as usize]
            .as_ref()
            .and_then(|node| node.get(key, current_key_offset + 1))
    }

    pub(crate) fn insert(
        mut self,
        key: &[u8; 32],
        value: V,
        current_key_offset: usize,
    ) -> (Self, Option<V>) {
        let mut old_value = None;
        self.choices[KeySegmentIterator::nth(key, current_key_offset) as usize] = Some(match self
            .choices[KeySegmentIterator::nth(key, current_key_offset) as usize]
            .take()
        {
            Some(mut x) => {
                let new_node;
                (new_node, old_value) = x.insert(key, value, current_key_offset + 1);
                *x = new_node;
                x
            }
            None => Box::new(LeafNode::from_key_value(key.to_owned(), value).into()),
        });

        (self, old_value)
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn remove(
        mut self,
        key: &[u8; 32],
        current_key_offset: usize,
    ) -> (Option<(Option<u8>, Node<V>)>, Option<V>) {
        let index = KeySegmentIterator::nth(key, current_key_offset) as usize;
        match self.choices[index].take() {
            Some(mut child_node) => {
                let (new_child, old_value) = child_node.remove(key, current_key_offset + 1);
                if let Some(new_child) = new_child {
                    *child_node = new_child;
                    self.choices[index] = Some(child_node);
                }

                let mut single_child = None;
                for (index, child) in self.choices.iter_mut().enumerate() {
                    if child.is_none() {
                        continue;
                    }

                    match single_child {
                        Some(_) => return (Some((None, self.into())), old_value),
                        None => single_child = Some((index, child)),
                    }
                }

                (
                    match single_child {
                        Some((index, x)) => match x.take() {
                            Some(x) => Some((Some(index as u8), *x)),
                            None => unreachable!(),
                        },
                        None => None,
                    },
                    old_value,
                )
            }
            None => (Some((None, self.into())), None),
        }
    }

    pub(crate) fn drain_filter<F>(
        mut self,
        filter: &mut F,
        drained_items: &mut Vec<([u8; 32], V)>,
        current_key_offset: usize,
    ) -> Option<(Option<u8>, Node<V>)>
    where
        F: FnMut(&[u8; 32], &mut V) -> bool,
    {
        enum SingleChild {
            NoChildren,
            Single(usize),
            Multiple,
        }

        let mut single_child = SingleChild::NoChildren;
        for (index, choice) in self.choices.iter_mut().enumerate() {
            *choice = match choice.take() {
                Some(old_node) => old_node
                    .drain_filter(filter, drained_items, current_key_offset + 1)
                    .map(|new_node| {
                        single_child = match single_child {
                            SingleChild::NoChildren => SingleChild::Single(index),
                            _ => SingleChild::Multiple,
                        };

                        Box::new(new_node)
                    }),
                None => None,
            };
        }

        match single_child {
            SingleChild::NoChildren => None,
            SingleChild::Single(index) => match self.choices[index].take() {
                Some(x) => Some((Some(index as u8), *x)),
                None => unreachable!(),
            },
            SingleChild::Multiple => Some((None, self.into())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{pm_tree_branch, pm_tree_key};

    #[test]
    fn from_choices() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node = BranchNode::from_choices({
            let mut choices: [Option<Box<Node<i32>>>; 16] = Default::default();

            choices[0] = Some(Box::new(
                pm_tree_branch! {
                    leaf { key_a => 42 }
                }
                .into(),
            ));
            choices[1] = Some(Box::new(
                pm_tree_branch! {
                    leaf { key_b => 43 }
                }
                .into(),
            ));

            choices
        });

        assert_eq!(
            node,
            pm_tree_branch! {
                branch {
                    0 => leaf { key_a => 42 },
                    1 => leaf { key_b => 43 },
                }
            },
        );
    }

    #[test]
    fn choices() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        };

        assert_eq!(node.choices(), &{
            let mut choices: [Option<Box<Node<i32>>>; 16] = Default::default();

            choices[0] = Some(Box::new(
                pm_tree_branch! {
                    leaf { key_a => 42 }
                }
                .into(),
            ));
            choices[1] = Some(Box::new(
                pm_tree_branch! {
                    leaf { key_b => 43 }
                }
                .into(),
            ));

            choices
        });
    }

    #[test]
    fn get_some() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        };

        assert_eq!(node.get(&key_a, 0), Some(&42));
        assert_eq!(node.get(&key_b, 0), Some(&43));
    }

    #[test]
    fn get_none() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");
        let key_c =
            pm_tree_key!("2000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        };

        assert_eq!(node.get(&key_c, 0), None);
    }

    #[test]
    fn insert() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");
        let key_c =
            pm_tree_key!("2000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        };

        let (node, old_value) = node.insert(&key_c, 44, 0);

        assert_eq!(old_value, None);
        assert_eq!(
            node,
            pm_tree_branch! {
                branch {
                    0 => leaf { key_a => 42 },
                    1 => leaf { key_b => 43 },
                    2 => leaf { key_c => 44 },
                }
            },
        );
    }

    #[test]
    fn insert_override() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        };

        let (node, old_value) = node.insert(&key_b, 44, 0);

        assert_eq!(old_value, Some(43));
        assert_eq!(
            node,
            pm_tree_branch! {
                branch {
                    0 => leaf { key_a => 42 },
                    1 => leaf { key_b => 44 },
                }
            },
        );
    }

    #[test]
    fn remove() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");
        let key_c =
            pm_tree_key!("2000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
                2 => leaf { key_c => 44 },
            }
        };

        let (node, old_value) = node.remove(&key_c, 0);

        assert_eq!(old_value, Some(44));
        assert_eq!(
            node,
            Some((
                None,
                pm_tree_branch! {
                    branch {
                        0 => leaf { key_a => 42 },
                        1 => leaf { key_b => 43 },
                    }
                }
                .into(),
            )),
        );
    }

    #[test]
    fn remove_simplify() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        };

        let (node, old_value) = node.remove(&key_b, 0);

        assert_eq!(old_value, Some(43));
        assert_eq!(
            node,
            Some((
                Some(0),
                pm_tree_branch! {
                    leaf { key_a => 42 }
                }
                .into(),
            )),
        );
    }

    #[test]
    fn remove_passthrough() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");
        let key_c =
            pm_tree_key!("0100000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => branch {
                    0 => leaf { key_a => 42 },
                    1 => leaf { key_c => 44 },
                },
                1 => leaf { key_b => 43 },
            }
        };

        let (node, old_value) = node.remove(&key_c, 0);

        assert_eq!(old_value, Some(44));
        assert_eq!(
            node,
            Some((
                None,
                pm_tree_branch! {
                    branch {
                        0 => leaf { key_a => 42 },
                        1 => leaf { key_b => 43 },
                    }
                }
                .into(),
            )),
        );
    }

    #[test]
    fn remove_nonexistent() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");
        let key_c =
            pm_tree_key!("2000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        };

        let (node, old_value) = node.remove(&key_c, 0);

        assert_eq!(old_value, None);
        assert_eq!(
            node,
            Some((
                None,
                pm_tree_branch! {
                    branch {
                        0 => leaf { key_a => 42 },
                        1 => leaf { key_b => 43 },
                    }
                }
                .into(),
            )),
        );
    }

    #[test]
    fn drain_filter() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");
        let key_c =
            pm_tree_key!("2000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
                2 => leaf { key_c => 44 },
            }
        };

        let mut drained_items = Vec::new();
        let node = node.drain_filter(&mut |key, _| key == &key_c, &mut drained_items, 0);

        assert_eq!(
            node,
            Some((
                None,
                pm_tree_branch! {
                    branch {
                        0 => leaf { key_a => 42 },
                        1 => leaf { key_b => 43 },
                    }
                }
                .into(),
            )),
        );
        assert_eq!(&drained_items, &[(key_c, 44)]);
    }

    #[test]
    fn drain_filter_simplify() {
        let key_a =
            pm_tree_key!("0000000000000000000000000000000000000000000000000000000000000000");
        let key_b =
            pm_tree_key!("1000000000000000000000000000000000000000000000000000000000000000");

        let node = pm_tree_branch! {
            branch {
                0 => leaf { key_a => 42 },
                1 => leaf { key_b => 43 },
            }
        };

        let mut drained_items = Vec::new();
        let node = node.drain_filter(&mut |key, _| key == &key_b, &mut drained_items, 0);

        assert_eq!(
            node,
            Some((
                Some(0),
                pm_tree_branch! {
                    leaf { key_a => 42 }
                }
                .into(),
            )),
        );
        assert_eq!(&drained_items, &[(key_b, 43)]);
    }
}
