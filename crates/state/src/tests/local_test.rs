use crate::{
    traits::StataTrait, Blake2bHasher, Data, OptimisticTransactionDB, Result, SmtValue, State, H256,
};
use rocksdb::prelude::Open;

fn new_state() -> State<'static, Blake2bHasher> {
    let db = OptimisticTransactionDB::open_default("./db1").unwrap();
    let prefix = b"test";
    State::new(prefix, db)
}

#[test]
fn main() {
    let mut state = new_state();
    // fixme no pass
    println!("value: {:?}", state.try_get(H256::from([0u8; 32])).unwrap());
    state.try_clear().unwrap();
    println!(
        "value(after clear): {:?}",
        state.try_get(H256::from([0u8; 32])).unwrap()
    );
    let kvs = vec![(
        H256::from([0u8; 32]),
        Data {
            address: Default::default(),
            nonce: 1,
            deposit: Default::default(),
        },
    )];
    let p_root = state.try_get_root().unwrap();
    println!("before update root: {:?}", p_root);

    // assert_eq!(state.try_get_root().unwrap(), H256::from([0u8; 32]));
    let root = state.try_update_all(kvs).unwrap();
    println!("update root1: {:?}", root);
    assert_ne!(root, H256::from([0u8; 32]));
    let proof = state
        .try_get_merkle_proof(vec![H256::from([0u8; 32])])
        .unwrap();
    let f_root = state
        .try_get_future_root(
            proof,
            vec![(
                H256::from([0u8; 32]),
                Data {
                    address: Default::default(),
                    nonce: 2,
                    deposit: Default::default(),
                },
            )],
        )
        .unwrap();
    assert_ne!(root, f_root);
    let ff_root = state
        .try_update_all(vec![(
            H256::from([0u8; 32]),
            Data {
                address: Default::default(),
                nonce: 2,
                deposit: Default::default(),
            },
        )])
        .unwrap();
    println!("update root2:{:?}", ff_root);
    assert_eq!(f_root, ff_root);
    let k = state.try_get(H256::from([0u8; 32])).unwrap().unwrap();
    assert_eq!(k.nonce, 2);
    ///
    state
        .try_update_all(vec![(H256::from([0u8; 32]), Default::default())])
        .unwrap();
    assert_eq!(state.try_get_root().unwrap(), H256::zero());

    let fff_root = state
        .try_update_all(vec![(
            H256::from([0u8; 32]),
            Data {
                address: Default::default(),
                nonce: 2,
                deposit: Default::default(),
            },
        )])
        .unwrap();
    assert_ne!(fff_root, H256::zero());
}
