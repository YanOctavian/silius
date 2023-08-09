//! This crate maintains Bundler's off-chain state.
//!
//! Usually, in projects related to zero-knowledge proof,
//! some data that originally relied on the chain to be stored can be stored off-chain.
//! In the ERC4337 protocol, we recommend storing the nonce value and deposit on the chain off the chain,
//! so that we can complete the deduction of Gas fee and modification of the Nonce value off the chain,
//! which will greatly reduce the Gas cost.
//!
//! State should not be shared between different chains, each chain has its own state.
//!
//! For the decentralization of bundlers, we will allow anyone to create their bundlers.
//! Before starting the bundler program, the first step you should do is to synchronize the
//! state off the chain to ensure that the Merkle root of your state is the same as that of others,
//! and further verify the legitimacy of the Merkle root stored on the chain.
//! This requires that the details of your synchronization process must update the state in a certain order.
//!

mod tests;
mod traits;

use std::hash::Hasher;
use smt_rocksdb_store::default_store::DefaultStoreMultiTree;
use sparse_merkle_tree::{H256, SparseMerkleTree, traits::Value};
use sparse_merkle_tree::blake2b::Blake2bHasher;
use ethers::types::{Address, U256};
use blake2b_rs::{Blake2b, Blake2bBuilder};


// use traits::StataTrait;
// //
// // #[derive(Debug, Clone)]
// // pub struct SmtValue {
// //     pub address: Address,
// //     pub nonce: u64,
// //     pub deposit: U256,
// // }
// //
// impl Value for SmtValue {
//     fn to_h256(&self) -> H256 {
//         let mut hasher = new_blake2b();
//         let mut buf = [0u8; 32];
//         hasher.update(&self.address.as_bytes());
//         hasher.update(&self.nonce.to_le_bytes());
//         hasher.update(&self.deposit.to_le_bytes());
//         hasher.finalize(&mut buf);
//         buf.into()
//     }
//
//     fn zero() -> Self {
//         Default::default()
//     }
// }

//
//
// type DefaultStoreMultiSMT<'a, T, W> =
//     SparseMerkleTree<Blake2bHasher, SmtValue, DefaultStoreMultiTree<'a, T, W>>;
//
//
// #[derive(Debug, Clone)]
// pub struct State<'a, S, H> {
//     prefix: &'a str,
//     db: S,
//     hasher: H,
// }
//
// impl<S: Default, H: Hasher + Default> State<'static, S, H> {
//     pub fn new(prefix: &'static str) -> Self {
//         Self {
//             prefix,
//             store: S::default(),
//             hasher: H::default(),
//         }
//     }
// }
//
//
// fn new_blake2b() -> Blake2b {
//     Blake2bBuilder::new(32).personal(b"SMT").build()
// }
//
//
// // impl StataTrait<H256, H256> for State<S, H> {
// //     fn try_update_all(&mut self, future_k_v: Vec<(H256, H256)>) -> anyhow::Result<Vec<H256>> {
// //         todo!()
// //     }
// //
// //     fn try_clear(&mut self) -> anyhow::Result<()> {
// //         todo!()
// //     }
// //
// //     fn try_get_merkle_proof(&self, keys: Vec<H256>) -> anyhow::Result<Vec<u8>> {
// //         todo!()
// //     }
// //
// //     fn try_get_future_root(&self, old_proof: Vec<u8>, future_k_v: Vec<(H256, H256)>) -> anyhow::Result<H256> {
// //         todo!()
// //     }
// // }
//
//
//
