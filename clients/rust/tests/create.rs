#![cfg(feature = "test-sbf")]

use borsh::BorshDeserialize;
use merkle_tree_storage::{accounts::MerkleTree, instructions::{CreateBuilder, InsertLeafBuilder}};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program, sysvar,
    transaction::Transaction,
};
use sha3::{Digest, Keccak256};

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

#[tokio::test]
async fn create() {
    let mut context =
        ProgramTest::new("merkle_tree_storage_program", merkle_tree_storage::ID, None)
            .start_with_context()
            .await;

    // Given a new keypair.
    let _address = Keypair::new();

    let (tree_pda, _bump) = Pubkey::find_program_address(
        &[b"tree", context.payer.pubkey().as_ref()],
        &merkle_tree_storage::ID,
    );

    let ix_create = CreateBuilder::new()
        .payer(context.payer.pubkey())
        .tree(tree_pda)
        .system_program(system_program::ID)
        .sysvar_rent(sysvar::rent::ID)
        .instruction();

    let ix_insert_first_leaf = InsertLeafBuilder::new()
        .payer(context.payer.pubkey())
        .tree(tree_pda)
        .leaf(keccak256(b"First leaf"))
        .instruction();

    let ix_insert_second_leaf = InsertLeafBuilder::new()
        .payer(context.payer.pubkey())
        .tree(tree_pda)
        .leaf(keccak256(b"Second leaf"))
        .instruction();

    // When we create a new account.
    let tx = Transaction::new_signed_with_payer(
        &[ix_create, ix_insert_first_leaf, ix_insert_second_leaf],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account was created with the correct data.

    let account = context.banks_client.get_account(tree_pda).await.unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    //assert_eq!(account.data.len(), MerkleTree::TREE_SIZE_BYTES);

    let mut account_data = account.data.as_ref();
    let my_account = MerkleTree::deserialize(&mut account_data).unwrap();
    assert_eq!(my_account.next_leaf_index, 2);
}
