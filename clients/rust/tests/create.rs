#![cfg(feature = "test-sbf")]

use borsh::BorshDeserialize;
use merkle_tree_storage::{
    accounts::MerkleTree,
    instructions::{CreateTreeBuilder, InsertLeafBuilder},
};
use sha3::{Digest, Keccak256};
use solana_program_test::{
    tokio,
    BanksClientError,
    ProgramTest,
    ProgramTestContext
};
use solana_sdk::{
    instruction::InstructionError,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::transfer,
    system_program,
    sysvar,
    transaction::{Transaction, TransactionError}
};

//use crate::off_chain_tree::OffchainMerkleTree;

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub struct SharedContext {
    context: ProgramTestContext,
    tree_pda: Pubkey
}

async fn get_context() -> SharedContext {
    let mut context =
        ProgramTest::new("merkle_tree_storage_program", merkle_tree_storage::ID, None)
            .start_with_context()
            .await;

    let (tree_pda, _bump) = Pubkey::find_program_address(
        &[b"tree", context.payer.pubkey().as_ref()],
        &merkle_tree_storage::ID,
    );

    let ix_create_tree = CreateTreeBuilder::new()
        .payer(context.payer.pubkey())
        .tree(tree_pda)
        .system_program(system_program::ID)
        .sysvar_rent(sysvar::rent::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix_create_tree],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context.banks_client.process_transaction(tx).await.expect("Cannot create tree PDA");
    let account = context.banks_client.get_account(tree_pda).await.expect("Unable get acount");
    assert!(account.is_some());

    SharedContext {
        context,
        tree_pda
    }
}

#[tokio::test]
async fn accountAccess() {
    let mut shared = get_context().await;
    // Given a new keypair.
    let hacker = Keypair::new();
    // send SOL from payer to hacker
    let ix_transfer = transfer(&shared.context.payer.pubkey(), &hacker.pubkey(), 100000000000000);

    let tx = Transaction::new_signed_with_payer(
        &[ix_transfer],
        Some(&shared.context.payer.pubkey()),
        &[&shared.context.payer],
        shared.context.last_blockhash,
    );
    shared.context.banks_client.process_transaction(tx).await.unwrap();


    let ix_insert_leaf = InsertLeafBuilder::new()
        .payer(hacker.pubkey())
        .tree(shared.tree_pda)
        .leaf(keccak256(&[1; 32]))
        .instruction();


    let tx = Transaction::new_signed_with_payer(
        &[ix_insert_leaf],
        Some(&hacker.pubkey()),
        &[&hacker],
        shared.context.last_blockhash,
    );
    
    //process transaction and expect custom error with code 4
    let error = shared.context.banks_client.process_transaction(tx).await.unwrap_err();
    match error {
        BanksClientError::TransactionError(TransactionError::InstructionError(index, error)) => {
            println!("❌ Instruction {} failed with error: {:?}", index, error);
    
            if let InstructionError::Custom(code) = error {
                assert_eq!(code, 4);
                println!("🔍 Custom program error code: 0x{:X} (decimal: {})", code, code);
            } else {
                panic!("Expected custom error with code 4");
            }
        }
        _ => {
            panic!("Expected custom error with code 4");
        }
    }
}
#[tokio::test]
async fn insert_leaf() {
    let mut shared = get_context().await;

    let ix_insert_first_leaf = InsertLeafBuilder::new()
        .payer(shared.context.payer.pubkey())
        .tree(shared.tree_pda)
        .leaf([1; 32])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix_insert_first_leaf],
        Some(&shared.context.payer.pubkey()),
        &[&shared.context.payer],
        shared.context.last_blockhash,
    );
    shared.context.banks_client.process_transaction(tx).await.unwrap();

    let tree_pda = shared.tree_pda.clone();
    let account = shared.context.banks_client.get_account(tree_pda).await.expect("Unable get acount");
    assert!(account.is_some());
    let account = account.unwrap();

    let mut account_data = account.data.as_ref();
    let my_account = MerkleTree::deserialize(&mut account_data).unwrap();
    assert_eq!(my_account.next_leaf_index, 1);

    //let mut tree = OffchainMerkleTree {
    //    nodes: vec![[0; 32]; OffchainMerkleTree::TREE_SIZE],
    //    next_leaf_index: 0,
    //};

    //tree.insert_leaf([1; 32]).unwrap();

    //let root = tree.nodes[0];
    //println!("📦 Merkle Root: {}", hex::encode(root));
    //assert_eq!(root, my_account.nodes[0]);
}

