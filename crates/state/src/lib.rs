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
use smt_rocksdb_store;
