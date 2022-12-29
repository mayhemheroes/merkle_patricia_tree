pub struct KeySegmentIterator<'a> {
    data: &'a [u8; 32],
    pos: usize,
    half: bool,
}

impl<'a> KeySegmentIterator<'a> {
    /// Create a new nibble iterator.
    pub fn new(data: &'a [u8; 32]) -> Self {
        Self {
            data,
            pos: 0,
            half: false,
        }
    }

    /// Shortcut to the `nth()` method of a new iterator.
    ///
    /// Panics when n is out of the range [0, 64).
    pub fn nth(data: &'a [u8; 32], n: usize) -> u8 {
        KeySegmentIterator::new(data)
            .nth(n)
            .expect("Key index out of range, value should be in [0, 64).")
    }
}

impl<'a> Iterator for KeySegmentIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= 32 {
            return None;
        }

        let mut value = self.data[self.pos];

        if self.half {
            self.pos += 1;
            value &= 0xF;
        } else {
            value >>= 4;
        }

        self.half = !self.half;
        Some(value)
    }
}

/// Create a new Patricia tree key.
#[cfg(test)]
#[macro_export]
macro_rules! pm_tree_key {
    ( $key:literal ) => {{
        assert_eq!($key.len(), 64, "Tree keys must be 64 nibbles in length.");
        let key: [u8; 32] = $key
            .as_bytes()
            .chunks_exact(2)
            .map(|x| {
                u8::from_str_radix(std::str::from_utf8(x).unwrap(), 16)
                    .expect("Key contains non-hexadecimal characters.")
            })
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap();

        key
    }};
}

/// Create a new Patricia Tree.
#[cfg(test)]
#[macro_export]
macro_rules! pm_tree {
    // Create an empty tree (with deduced value type).
    () => {
        $crate::PatriciaTree {
            root_node: None,
        }
    };
    // Create an empty tree (with explicit value type).
    ( < $t:ty > ) => {
        $crate::PatriciaTree::<$t> {
            root_node: None,
        }
    };
    // Create a new tree.
    ( $type:ident { $( $root_node:tt )* } ) => {
        $crate::PatriciaTree {
            root_node: Some($crate::pm_tree_branch!($type { $( $root_node )* }).into()),
        }
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! pm_tree_branch {
    // Internal.
    ( branch { $( $key:literal => $type:ident { $( $node:tt )* } ),* $(,)? } ) => {
        $crate::node::BranchNode::from_choices({
            let mut choices: [Option<Box<Node<_>>>; 16] = Default::default();
            $( choices[$key] = Some(Box::new($crate::pm_tree_branch!($type { $( $node )* }).into())); )*
            choices
        })
    };
    // Internal.
    ( extension { $prefix:literal, $type:ident { $( $node:tt )* } } ) => {
        $crate::node::ExtensionNode::from_prefix_child(
            {
                let value = $prefix
                    .as_bytes()
                    .into_iter()
                    .map(|x| {
                        (*x as char)
                            .to_digit(16)
                            .expect("Prefix contains non-hexadecimal characters.") as u8
                    })
                    .collect::<Vec<u8>>();

                value
            },
            $crate::pm_tree_branch!($type { $( $node )* }).into(),
        )
    };
    // Internal.
    ( leaf { $key:expr => $value:expr } ) => {
        $crate::node::LeafNode::from_key_value($key, $value)
    };
}
