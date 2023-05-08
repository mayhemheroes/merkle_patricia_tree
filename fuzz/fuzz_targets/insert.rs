#![no_main]
use libfuzzer_sys::fuzz_target;
use patricia_merkle_tree::PatriciaMerkleTree;
use sha3::Keccak256;

fuzz_target!(|value: Vec<(String,String)>| {
    let mut tree = PatriciaMerkleTree::<String, String, Keccak256>::new();
    let mut data = value;
    data.sort_by(|a,b| a.0.len().cmp(&b.0.len()));
    for pair in data {
        tree.insert(pair.0, pair.1);
    }
});
