//!
//! Just a feasibility test about sparse Merkle trees.
//!
//! For example, is it possible to modify many keys at one time,
//! and is it possible to obtain a modified new Merkle root based on the old Merkle proof.
//!

use blake2b_rs::{Blake2b, Blake2bBuilder};
use sparse_merkle_tree::{
    blake2b::Blake2bHasher, default_store::DefaultStore, traits::Value, SparseMerkleTree, H256,
};

// define SMT
type SMT = SparseMerkleTree<Blake2bHasher, Word, DefaultStore<Word>>;

// define SMT value
#[derive(Default, Clone)]
pub struct Word(String);
impl Value for Word {
    fn to_h256(&self) -> H256 {
        if self.0.is_empty() {
            return H256::zero();
        }
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(self.0.as_bytes());
        hasher.finalize(&mut buf);
        buf.into()
    }
    fn zero() -> Self {
        Default::default()
    }
}

// helper function
fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32).personal(b"SMT").build()
}

fn get_key(k: u32) -> H256 {
    let key: H256 = {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&k.to_le_bytes());
        hasher.finalize(&mut buf);
        buf.into()
    };
    key
}

/// Test whether it is possible to get a new Merkle root without updating the current state.
fn get_new_root_should_work(
    tree: &mut SMT,
    old_leaves: Vec<(H256, Word)>,
    new_leaves: Vec<(H256, Word)>,
) {
    let leaves = old_leaves
        .iter()
        .map(|(k, v)| (*k, v.to_h256()))
        .collect::<Vec<_>>();

    let new_leaves_cp = new_leaves.clone();
    let new_leavse = new_leaves
        .iter()
        .map(|(k, v)| (*k, v.to_h256()))
        .collect::<Vec<_>>();

    let key1_proof = tree
        .merkle_proof(leaves.clone().iter().map(|(k, _)| *k).collect())
        .unwrap();
    let root = tree.root();
    let first_v = key1_proof
        .clone()
        .verify::<Blake2bHasher>(root, leaves.clone())
        .unwrap();
    assert!(first_v, "The first verification not passed");

    let futrue_root = key1_proof
        .clone()
        .compute_root::<Blake2bHasher>(new_leavse.clone())
        .unwrap();

    let second_v = key1_proof
        .clone()
        .verify::<Blake2bHasher>(&futrue_root, leaves.clone())
        .unwrap();
    assert!(
        !second_v,
        "the verification passes after modifying the Merkle root"
    );

    new_leaves_cp.iter().for_each(|(k, v)| {
        let _ = tree.update(*k, v.clone()).unwrap();
    });

    let new_key_proof = tree
        .merkle_proof(new_leaves.clone().iter().map(|(k, _)| *k).collect())
        .unwrap();
    let three_v = new_key_proof
        .verify::<Blake2bHasher>(&futrue_root, new_leavse)
        .unwrap();
    assert!(
        three_v,
        "the verification not passes after updating the data"
    );
}

#[test]
fn main() {
    let mut tree = SMT::default();
    for (i, word) in "The quick brown fox jumps over the lazy dog"
        .split_whitespace()
        .enumerate()
    {
        let key: H256 = {
            let mut buf = [0u8; 32];
            let mut hasher = new_blake2b();
            hasher.update(&(i as u32).to_le_bytes());
            hasher.finalize(&mut buf);
            buf.into()
        };
        let value = Word(word.to_string());
        // insert key value into tree
        tree.update(key, value).expect("update");
    }

    get_new_root_should_work(
        &mut tree,
        vec![(get_key(0), Word("The".to_string()))],
        vec![(get_key(0), Word("wow".to_string()))],
    );

    let old = vec![
        (get_key(0), Word("wow".to_string())),
        (get_key(1), Word("quick".to_string())),
    ];
    let new = vec![
        (get_key(0), Word("wow1".to_string())),
        (get_key(1), Word("quick1".to_string())),
    ];
    get_new_root_should_work(&mut tree, old, new);
}
