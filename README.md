# **WIP README**. 

We will be updating the README in the following days.

# Patricia Tree implementation in Rust

This crate contains an implementation of a Patricia Tree.

Its structure is implemented to match Ethereum's [Patricia Merkle Trees](https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie/),
having the key hardcoded to 32 bytes, and each branch node keeping track of a
single nibble. It does not contain neither storage nor the hashing function.

> TODO: Storage implementation example.

# Benchmarking

Benchmarks are provided for the following use cases:

  - Retrieval of non-existant nodes.
  - Retrieval of existing nodes.
  - Insertion of new nodes.
  - Overwriting nodes.
  - Removal of nodes.
  - Removal of non-existing nodes (no-op).

Every use case is tested with different tree sizes, ranging from 1k to 1M.

# What is a patricia merke tree

PATRICIA is an acronym which means:

Practical Algorithm to Retrieve Information Coded in Alphanumeric

> A compact representation of a trie in which any node that is an only child is merged with its parent. 

> Patricia tries are seen as radix trees with radix equals 2, which means that each bit of the key is compared individually and each node is a two-way (i.e., left versus right) branch

In essence, a patricia tree stores a value for the given key (also named **path**).

The key or path is converted to bytes, and then each nibble of each byte is used to traverse the tree.

It is composed of 3 different types of nodes:

## The branch node

It contains a 17 element array:
- The 16 first elements cover every representable value of a nibble (2^4 = 16)
- The value in case the path is fully traversed.

## The leaf node

It contains 2 elements:
- The encoded path.
- The value.

## The extension node

It contains 2 elements:
- The encoded path.
- A key or path for the next lookup.

This node allows the tree to be more compact, imagine we have a path that ultimately can only go 1 way, because it has no diverging paths,
adding X nodes instead of 1 representing that would be a waste of space, this fixes that.

For example, imagine we have the keys "abcdx" and "abcdy", instead of adding 10 nodes (1 for each nibble in each character), we create a single node representing the path "abcd", thus compressing the tree.


## Solving the ambiguity

Since traversing a path is done through it's nibbles, when doing so, the remaining partial path may have an odd number of nibbles left, this
introduces an ambiguity that comes from storing a nibble as a byte:

Imagine we have the following remaining nibbles:

- `1`
- `01`

When representing both as a byte, they have the same value `1`.

Thats why a flag is introduced to differenciate between an odd or even remaining partial path:

TODO: put the flag table


## Terms
- **[nibble](https://en.wikipedia.org/wiki/Nibble)**: 4bits, half a byte, a single hex digit.

## Useful links

- [Patricia tree on NIST](https://xlinux.nist.gov/dads/HTML/patriciatree.html)
- [Paper by Donald R. Morrison](https://dl.acm.org/doi/10.1145/321479.321481)
