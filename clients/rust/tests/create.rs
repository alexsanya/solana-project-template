#![cfg(feature = "test-sbf")]

use borsh::BorshDeserialize;
use merkle_tree_storage::{
    accounts::MerkleTree,
    instructions::{CreateTreeBuilder, InsertLeafBuilder},
};
use sha2::Sha256;
use sha3::{Digest, Keccak256};
use solana_program_test::{
    tokio::{self, sync::Mutex, sync::OnceCell},
    BanksClientError, ProgramTest, ProgramTestContext,
};
use solana_sdk::{
    instruction::{Instruction, InstructionError},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::transfer,
    system_program, sysvar,
    transaction::{Transaction, TransactionError},
};

mod off_chain_tree;
use off_chain_tree::OffchainMerkleTree;

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub struct SharedContext {
    context: ProgramTestContext,
    tree_pda: Pubkey,
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
        .max_depth(3)
        .system_program(system_program::ID)
        .sysvar_rent(sysvar::rent::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix_create_tree],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(tx)
        .await
        .expect("Cannot create tree PDA");
    let account = context
        .banks_client
        .get_account(tree_pda)
        .await
        .expect("Unable get acount");
    assert!(account.is_some());

    SharedContext { context, tree_pda }
}

#[tokio::test]
async fn prevent_insert_leaf_to_wrong_account() {
    let mut shared = get_context().await;
    // Given a new keypair.
    let hacker = Keypair::new();
    // send SOL from payer to hacker
    let ix_transfer = transfer(
        &shared.context.payer.pubkey(),
        &hacker.pubkey(),
        100000000000000,
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix_transfer],
        Some(&shared.context.payer.pubkey()),
        &[&shared.context.payer],
        shared.context.last_blockhash,
    );
    shared
        .context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();

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
    let error = shared
        .context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    if let BanksClientError::TransactionError(TransactionError::InstructionError(
        index,
        InstructionError::Custom(code),
    )) = error
    {
        println!(
            "Instruction {} failed with custom error code: 0x{:X}",
            index, code
        );
        assert_eq!(code, 4);
    } else {
        panic!("Expected custom error with code 4");
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
    shared
        .context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    let tree_pda = shared.tree_pda.clone();
    let account = shared
        .context
        .banks_client
        .get_account(tree_pda)
        .await
        .expect("Unable get acount");
    assert!(account.is_some());
    let account = account.unwrap();

    let mut account_data = account.data.as_ref();
    let my_account = MerkleTree::deserialize(&mut account_data).unwrap();
    assert_eq!(my_account.next_leaf_index, 1);

    let mut tree = OffchainMerkleTree {
        nodes: vec![[0; 32]; OffchainMerkleTree::TREE_SIZE],
        next_leaf_index: 0,
    };

    tree.insert_leaf([1; 32]).unwrap();

    let root = tree.nodes[0];
    assert_eq!(root, my_account.nodes[0]);
}

#[tokio::test]
async fn insert_maximum_leafs() {
    let mut shared = get_context().await;

    let leaves = vec![
        keccak256(b"First"),
        keccak256(b"Second"),
        keccak256(b"Third"),
        keccak256(b"Fourth"),
        keccak256(b"Fifth"),
        keccak256(b"Sixth"),
        keccak256(b"Seventh"),
        keccak256(b"Eighth"),
    ];

    let build_insert_leaf_ix = |leaf: [u8; 32]| {
        InsertLeafBuilder::new()
            .payer(shared.context.payer.pubkey())
            .tree(shared.tree_pda)
            .leaf(leaf)
            .instruction()
    };

    let ixs: Vec<Instruction> = leaves.iter().map(|leaf| build_insert_leaf_ix(*leaf)).collect();
    let tx = Transaction::new_signed_with_payer(
        &ixs[0..8],
        Some(&shared.context.payer.pubkey()),
        &[&shared.context.payer],
        shared.context.last_blockhash,
    );
    shared
        .context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap();

    let tree_pda = shared.tree_pda.clone();
    let account = shared
        .context
        .banks_client
        .get_account(tree_pda)
        .await
        .expect("Unable get acount");
    assert!(account.is_some());
    let account = account.unwrap();

    let mut account_data = account.data.as_ref();
    let my_account = MerkleTree::deserialize(&mut account_data).unwrap();
    assert_eq!(my_account.next_leaf_index, leaves.len() as u8);

    let mut tree = OffchainMerkleTree {
        nodes: vec![[0; 32]; OffchainMerkleTree::TREE_SIZE],
        next_leaf_index: 0,
    };

    for leaf in leaves {
        tree.insert_leaf(leaf).unwrap();
    }

    let root = tree.nodes[0];
    assert_eq!(root, my_account.nodes[0]);
}


#[tokio::test]
async fn overflow_tree() {
    let mut shared = get_context().await;

    let leaves = vec![
        keccak256(b"First"),
        keccak256(b"Second"),
        keccak256(b"Third"),
        keccak256(b"Fourth"),
        keccak256(b"Fifth"),
        keccak256(b"Sixth"),
        keccak256(b"Seventh"),
        keccak256(b"Eighth"),
        keccak256(b"Ninth")
    ];

    let build_insert_leaf_ix = |leaf: [u8; 32]| {
        InsertLeafBuilder::new()
            .payer(shared.context.payer.pubkey())
            .tree(shared.tree_pda)
            .leaf(leaf)
            .instruction()
    };

    let ixs: Vec<Instruction> = leaves.iter().map(|leaf| build_insert_leaf_ix(*leaf)).collect();
    let tx = Transaction::new_signed_with_payer(
        &ixs[0..9],
        Some(&shared.context.payer.pubkey()),
        &[&shared.context.payer],
        shared.context.last_blockhash,
    );

    let error = shared
        .context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err();
    if let BanksClientError::TransactionError(TransactionError::InstructionError(
        index,
        InstructionError::Custom(code),
    )) = error
    {
        println!(
            "Instruction {} failed with custom error code: 0x{:X}",
            index, code
        );
        assert_eq!(code, 3);
    } else {
        panic!("Expected custom error with code 4");
    }
}
