#![cfg(feature = "test-integration")]

use borsh::BorshDeserialize;
use merkle_tree_storage::{
    accounts::MerkleTree,
    instructions::{CreateTreeBuilder, InsertLeafBuilder},
};
use sha2::Sha256;
use sha3::{Digest, Keccak256};
use solana_client::rpc_client::RpcClient;
use solana_program_test::{
    tokio::{self, sync::Mutex, sync::OnceCell},
    BanksClientError, ProgramTest, ProgramTestContext,
};
use solana_sdk::commitment_config::CommitmentConfig;
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
    client: RpcClient,
    payer: Keypair,
    tree_pda: Pubkey,
}

async fn get_context() -> SharedContext {
    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );

    let payer = read_keypair_file("~/.config/solana/id.json").expect("Missing keypair");

    let (tree_pda, _bump) = Pubkey::find_program_address(
        &[b"tree", payer.pubkey().as_ref()],
        &merkle_tree_storage::ID,
    );

    SharedContext {
        client,
        payer,
        tree_pda,
    }
}

#[tokio::test]
async fn accountAccess() {
    let mut shared = get_context().await;
    // Given a new keypair.
    let hacker = Keypair::new();
    // send SOL from payer to hacker
    let ix_transfer = transfer(&shared.payer.pubkey(), &hacker.pubkey(), 100000000000000);

    let tx = Transaction::new_signed_with_payer(
        &[ix_transfer],
        Some(&shared.payer.pubkey()),
        &[&shared.payer],
        shared.client.get_last_blockhash().await.unwrap(),
    );
    shared
        .client
        .send_and_confirm_transaction(tx)
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
        shared.client.get_last_blockhash().await.unwrap(),
    );

    //process transaction and expect custom error with code 4
    let error = shared
        .client
        .send_and_confirm_transaction(tx)
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
        .payer(shared.payer.pubkey())
        .tree(shared.tree_pda)
        .leaf([1; 32])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix_insert_first_leaf],
        Some(&shared.payer.pubkey()),
        &[&shared.payer],
        shared.client.get_last_blockhash().await.unwrap(),
    );
    shared
        .client
        .send_and_confirm_transaction(tx)
        .await
        .unwrap();

    let tree_pda = shared.tree_pda.clone();
    let account = shared.client.get_account(tree_pda).await.unwrap();
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
async fn insert_multiple_leafs() {
    let mut shared = get_context().await;

    let leaves = vec![
        keccak256(b"First"),
        keccak256(b"Second"),
        keccak256(b"Third"),
    ];

    let build_insert_leaf_tx = |leaf: [u8; 32]| {
        InsertLeafBuilder::new()
            .payer(shared.payer.pubkey())
            .tree(shared.tree_pda)
            .leaf(leaf)
            .instruction()
    };

    let tx = Transaction::new_signed_with_payer(
        &[
            build_insert_leaf_tx(leaves[0]),
            build_insert_leaf_tx(leaves[1]),
            build_insert_leaf_tx(leaves[2]),
        ],
        Some(&shared.payer.pubkey()),
        &[&shared.payer],
        shared.client.get_last_blockhash().await.unwrap(),
    );
    shared
        .client
        .send_and_confirm_transaction(tx)
        .await
        .unwrap();

    let tree_pda = shared.tree_pda.clone();
    let account = shared.client.get_account(tree_pda).await.unwrap();
    assert!(account.is_some());
    let account = account.unwrap();

    let mut account_data = account.data.as_ref();
    let my_account = MerkleTree::deserialize(&mut account_data).unwrap();
    assert_eq!(my_account.next_leaf_index, 3);

    let mut tree = OffchainMerkleTree {
        nodes: vec![[0; 32]; OffchainMerkleTree::TREE_SIZE],
        next_leaf_index: 0,
    };

    tree.insert_leaf(leaves[0]).unwrap();
    tree.insert_leaf(leaves[1]).unwrap();
    tree.insert_leaf(leaves[2]).unwrap();

    let root = tree.nodes[0];
    assert_eq!(root, my_account.nodes[0]);
}
