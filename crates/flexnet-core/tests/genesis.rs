#[path = "shared/common.rs"]
mod common;

use flexnet_core::{Account, Genesis, Hash};

use self::common::{address_for, signing_key};

#[test]
fn parse_genesis_json_and_build_genesis_block() {
    let alice = address_for(&signing_key(1));
    let bob = address_for(&signing_key(2));
    let empty = address_for(&signing_key(3));
    let genesis_json = format!(
        r#"{{
  "chain_id": 1,
  "chain_version": 1,
  "block_height": 0,
  "previous_block_hash": "{zero_hash}",
  "state": {{
    "accounts": {{
      "{alice}": {{
        "balance": 1000,
        "nonce": 0
      }},
      "{bob}": {{
        "balance": 9999,
        "nonce": 0
      }},
      "{empty}": {{
        "balance": 0,
        "nonce": 0
      }}
    }}
  }}
}}"#,
        zero_hash = Hash::ZERO,
        alice = alice,
        bob = bob,
        empty = empty,
    );

    let genesis = Genesis::from_json_str(&genesis_json).unwrap();
    let block = genesis.block();

    assert_eq!(genesis.state.get_account(alice), Account::new(1_000, 0));
    assert_eq!(genesis.state.get_account(bob), Account::new(9_999, 0));
    assert_eq!(genesis.state.get_account(empty), Account::new(0, 0));
    assert_eq!(genesis.state.accounts().len(), 2);
    assert!(block.transactions.is_empty());
    assert_eq!(block.previous_block_hash, Hash::ZERO);
    assert_eq!(block.state_hash, genesis.state_hash());
    assert_ne!(genesis.block_hash(), Hash::ZERO);
}
