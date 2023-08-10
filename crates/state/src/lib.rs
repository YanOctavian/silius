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

#![allow(dead_code)]
// todo list
// 1. test for all functions.
// 2. hash algorithm can be replaced.
// 3. bundler calls state.
// 4. Bundler synchronizes data for state.
// 8. `thiserr` instead of `anyhow`.


mod tests;
mod traits;
pub mod utils;

use bincode;
use blake2b_rs::{Blake2b, Blake2bBuilder};
use byte_slice_cast::AsByteSlice;
use ethers::types::{Address, U256};
use rocksdb::prelude::{Iterate,};
use rocksdb::{DBVector, OptimisticTransaction, OptimisticTransactionDB,};
use rocksdb::{Direction, IteratorMode};
use serde::{Deserialize, Serialize};
use smt_rocksdb_store::default_store::DefaultStoreMultiTree;
use sparse_merkle_tree::{blake2b::Blake2bHasher, traits::Hasher, };
use sparse_merkle_tree::{traits::Value, CompiledMerkleProof, SparseMerkleTree, H256};
// local
use traits::StataTrait;
use utils::address_convert_to_h256;

type DefaultStoreMultiSMT<'a, H, T, W> =
    SparseMerkleTree<H, SmtValue, DefaultStoreMultiTree<'a, T, W>>;

fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32).personal(b"SMT").build()
}

/// The value stored in the sparse Merkle tree.
#[derive(Debug, Clone, Default)]
pub struct SmtValue {
    data: Data,
    serialized_data: Option<Vec<u8>>,
}

impl SmtValue {
    pub fn new(data: Data) -> Self {
        let serialized_data = bincode::serialize(&data).unwrap();
        SmtValue {
            data,
            serialized_data: Some(serialized_data),
        }
    }

    pub fn get_data(&self) -> &Data {
        &self.data
    }

    pub fn get_serialized_data(&self) -> &[u8] {
        self.serialized_data.as_ref().unwrap()
    }
}


#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Data {
    address: Address,
    nonce: u64,
    deposit: U256,
}

impl Value for SmtValue {
    fn to_h256(&self) -> H256 {
        let mut hasher = new_blake2b();
        let mut buf = [0u8; 32];
        hasher.update(self.get_serialized_data());
        hasher.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        Default::default()
    }
}

impl From<DBVector> for SmtValue {
    fn from(v: DBVector) -> Self {
        let data = bincode::deserialize(v.as_byte_slice()).unwrap();
        SmtValue {
            data,
            serialized_data: Some(v.to_vec()),
        }
    }
}

impl AsRef<[u8]> for SmtValue {
    fn as_ref(&self) -> &[u8] {
        self.get_serialized_data()
    }
}


/// The state of the bundler.
/// stores off-chain state, its merkle root is stored on-chain.
/// Each entry point contract of each chain has a state.
pub struct State<'a, H: Hasher + Default> {
    prefix: &'a [u8],
    db: OptimisticTransactionDB,
    hasher: H,
}


impl<'a, H: Hasher + Default> State<'a, H> {
    pub fn new(prefix: &'a [u8], db: OptimisticTransactionDB, hasher: H) -> Self {
        // todo Check if it has been created
        State { prefix, db, hasher }
    }

}


impl StataTrait<H256, Data> for State<'_, Blake2bHasher> {
    fn try_update_all(&mut self, future_k_v: Vec<(H256, Data)>) -> anyhow::Result<H256> {
        let kvs = future_k_v
            .into_iter()
            .map(|(k, v)| (k, SmtValue::new(v)))
            .collect::<Vec<_>>();
        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt: SparseMerkleTree<
            Blake2bHasher,
            SmtValue,
            DefaultStoreMultiTree<'_, OptimisticTransaction, ()>,
        > = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::new(self.prefix, &tx))?;
        rocksdb_store_smt.update_all(kvs)?;
        tx.commit()?;
        Ok(*rocksdb_store_smt.root())
    }

    fn try_clear(&mut self) -> anyhow::Result<()> {
        let snapshot = self.db.snapshot();
        let prefix = self.prefix;
        let prefix_len = prefix.len();
        let leaf_key_len = prefix_len + 32;
        let kvs: Vec<(H256, SmtValue)> = snapshot
            .iterator(IteratorMode::From(prefix, Direction::Forward))
            .take_while(|(k, _)| k.starts_with(prefix))
            .filter_map(|(k, _)| {
                if k.len() != leaf_key_len {
                    None
                } else {
                    let leaf_key: [u8; 32] = k[prefix_len..].try_into().expect("checked 32 bytes");
                    Some((leaf_key.into(), SmtValue::zero()))
                }
            })
            .collect();

        let tx = self.db.transaction_default();
        let mut rocksdb_store_smt: SparseMerkleTree<
            Blake2bHasher,
            SmtValue,
            DefaultStoreMultiTree<'_, OptimisticTransaction, ()>,
        > = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::new(
            prefix.as_byte_slice(),
            &tx,
        ))
        .unwrap();

        rocksdb_store_smt.update_all(kvs)?;
        tx.commit()?;
        assert_eq!(rocksdb_store_smt.root(), &H256::zero());
        Ok(())
    }

    fn try_get_merkle_proof(&self, keys: Vec<H256>) -> anyhow::Result<Vec<u8>> {
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt: SparseMerkleTree<
            Blake2bHasher,
            SmtValue,
            DefaultStoreMultiTree<'_, _, ()>,
        > = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            self.prefix,
            &snapshot,
        ))?;
        let proof = rocksdb_store_smt
            .merkle_proof(keys.clone())?
            .compile(keys)?;
        Ok(proof.0)
    }

    fn try_get_future_root(
        &self,
        old_proof: Vec<u8>,
        future_k_v: Vec<(H256, Data)>,
    ) -> anyhow::Result<H256> {
        let p = CompiledMerkleProof(old_proof);
        let kvs = future_k_v
            .into_iter()
            .map(|(k, v)| (k, SmtValue::new(v).to_h256()))
            .collect::<Vec<_>>();
        let f_root = p
            .compute_root::<Blake2bHasher>(kvs)
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(f_root)
    }

    fn try_get(&self, key: H256) -> anyhow::Result<Option<Data>> {
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt: SparseMerkleTree<
            Blake2bHasher,
            SmtValue,
            DefaultStoreMultiTree<'_, _, ()>,
        > = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            self.prefix,
            &snapshot,
        ))?;
        let v = rocksdb_store_smt.get(&key)?;
        let data = v.get_data();
        Ok(Some(data.clone()))
    }

    fn try_get_root(&self) -> anyhow::Result<H256> {
        let snapshot = self.db.snapshot();
        let rocksdb_store_smt: SparseMerkleTree<
            Blake2bHasher,
            SmtValue,
            DefaultStoreMultiTree<'_, _, ()>,
        > = DefaultStoreMultiSMT::new_with_store(DefaultStoreMultiTree::<_, ()>::new(
            self.prefix,
            &snapshot,
        ))?;
        let root = *rocksdb_store_smt.root();
        Ok(root)
    }
}
