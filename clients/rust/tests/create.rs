#![cfg(feature = "test-sbf")]

use borsh::BorshDeserialize;
use merkle_tree_storage::{accounts::MerkleTree, instructions::CreateBuilder};
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    signature::{Keypair, Signer}, system_program, sysvar, transaction::Transaction,
    pubkey::Pubkey
};

#[tokio::test]
async fn create() {
    let mut context =
        ProgramTest::new("merkle_tree_storage_program", merkle_tree_storage::ID, None)
            .start_with_context()
            .await;

    // Given a new keypair.

    let address = Keypair::new();

    let (tree_pda, bump) = Pubkey::find_program_address(&[b"tree", context.payer.pubkey().as_ref()], &merkle_tree_storage::ID);

    let ix = CreateBuilder::new()
        .payer(context.payer.pubkey())
        .tree(tree_pda)
        .system_program(system_program::ID)
        .sysvar_rent(sysvar::rent::ID)
        .instruction();

    // When we create a new account.

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.unwrap();

    // Then an account was created with the correct data.

    let account = context
        .banks_client
        .get_account(tree_pda)
        .await
        .unwrap();

    assert!(account.is_some());

    let account = account.unwrap();
    //assert_eq!(account.data.len(), MerkleTree::TREE_SIZE_BYTES);

    let mut account_data = account.data.as_ref();
    let my_account = MerkleTree::deserialize(&mut account_data).unwrap();
    assert_eq!(my_account.next_leaf_index, 0);
}
