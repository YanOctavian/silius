use super::Result;
use sparse_merkle_tree::H256;

/// Several basic implementations of off-chain state.
pub trait StataTrait<K, V> {
    /// update all data, and return the new root.
    fn try_update_all(&mut self, future_k_v: Vec<(K, V)>) -> Result<H256>;
    /// clear all data.
    fn try_clear(&mut self) -> Result<()>;
    /// get current merkle proof.
    fn try_get_merkle_proof(&self, keys: Vec<K>) -> Result<Vec<u8>>;
    /// ro get the future root without changing the state.
    fn try_get_future_root(&self, old_proof: Vec<u8>, future_k_v: Vec<(K, V)>) -> Result<H256>;
    /// get value by key.
    fn try_get(&self, key: K) -> Result<Option<V>>;
    /// get current merkle root.
    fn try_get_root(&self) -> Result<H256>;
}
