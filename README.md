# Patricia Tree implementation in Rust

This crate contains an implementation of a Patricia Tree.

Its structure is implemented to match Ethereum's [Patricia Merkle Trees](https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie/),
having the key hardcoded to 32 bytes, and each branch node keeping track of a
single nibble. It does not contain neither storage nor the hashing function.

> TODO: Storage implementation example.

## Benchmarking

Benchmarks are provided for the following use cases:

  - Retrieval of non-existant nodes.
  - Retrieval of existing nodes.
  - Insertion of new nodes.
  - Overwriting nodes.
  - Removal of nodes.
  - Removal of non-existing nodes (no-op).

Every use case is tested with different tree sizes, ranging from 1k to 1M.
