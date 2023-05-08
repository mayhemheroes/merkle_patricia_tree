#![no_main]
use libfuzzer_sys::fuzz_target;
use patricia_merkle_tree::PatriciaMerkleTree;
use sha3::Keccak256;

fuzz_target!(|value: Vec<(String,String)>| {
    let mut tree = PatriciaMerkleTree::<String, String, Keccak256>::new();
    for pair in value {
        tree.insert(pair.0, pair.1);
    }
    tree.compute_hash();
});
