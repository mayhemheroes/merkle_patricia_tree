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
