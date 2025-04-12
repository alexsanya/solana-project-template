use borsh::BorshDeserialize;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey,
    rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};
use hex;

use crate::error::MerkleTreeStorageError;
use crate::instruction::accounts::{CreateTreeAccounts, InsertLeafAccounts};
use crate::instruction::{CreateTreeArgs, InsertLeafArgs, MerkleTreeInstruction};
use crate::state::MerkleTree;

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: MerkleTreeInstruction =
        MerkleTreeInstruction::try_from_slice(instruction_data)?;
    match instruction {
        MerkleTreeInstruction::CreateTree(create_tree_args) => {
            msg!("Instruction: CreateTree");
            create_tree(program_id, accounts, create_tree_args)
        },
        MerkleTreeInstruction::InsertLeaf(insert_leaf_args) => {
            msg!("Instruction: InsertLeaf");
            insert_leaf(program_id, accounts, insert_leaf_args)
        }
    }
}

fn insert_leaf<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>], insert_leaf_args: InsertLeafArgs) -> ProgramResult {
    let ctx = InsertLeafAccounts::context(accounts)?;

    let (expected_pda, _bump) = Pubkey::find_program_address(
        &[b"tree", ctx.accounts.payer.key.as_ref()],
        program_id,
    );
    if &expected_pda != ctx.accounts.tree.key {
        msg!("Invalid tree account PDA for this payer");
        return Err(MerkleTreeStorageError::InvalidPDA.into());
    }
    if ctx.accounts.tree.owner != program_id {
        msg!("Invalid tree account owner");
        return Err(MerkleTreeStorageError::InvalidPDA.into());
    }
    if !ctx.accounts.payer.is_signer {
        msg!("Payer must be a signer");
        return Err(MerkleTreeStorageError::PayerMustBeSigner.into());
    }

    let mut tree = MerkleTree::load(ctx.accounts.tree)?;
    tree.insert_leaf(insert_leaf_args.leaf)?;
    tree.save(ctx.accounts.tree)?;
    msg!("Leaf inserted. Root: {}", hex::encode(tree.nodes[0]));
    Ok(())
}

fn create_tree<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>], create_tree_args: CreateTreeArgs) -> ProgramResult {
    // Accounts.
    let ctx = CreateTreeAccounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MerkleTreeStorageError::InvalidSystemProgram.into());
    }

    // Fetch the space and minimum lamports required for rent exemption.
    let space: usize = MerkleTree::get_tree_size_bytes(create_tree_args.max_depth);
    let lamports: u64 = rent.minimum_balance(space);
    let (expected_pda, bump) = Pubkey::find_program_address(&[b"tree", ctx.accounts.payer.key.as_ref()], program_id);
    if &expected_pda != ctx.accounts.tree.key {
        msg!("Invalid PDA provided");
        return Err(ProgramError::InvalidArgument);
    }

    // CPI to the System Program.
    invoke_signed(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.tree.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.tree.clone(),
            ctx.accounts.system_program.clone(),
        ],
        &[&[b"tree", ctx.accounts.payer.key.as_ref(), &[bump]]],
    )?;

    let tree = MerkleTree {
        nodes: vec![[0; 32]; MerkleTree::get_tree_size(create_tree_args.max_depth)],
        max_depth: create_tree_args.max_depth,
        next_leaf_index: 0,
    };

    tree.save(ctx.accounts.tree)?;

    msg!("Merkle tree PDA initialized");
    Ok(())
}
