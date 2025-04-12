use borsh::BorshDeserialize;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey,
    rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};
use hex;

use crate::error::MerkleTreeStorageError;
use crate::instruction::accounts::{CreateAccounts, InsertLeafAccounts};
use crate::instruction::{InsertLeafArgs, MerkleTreeInstruction};
use crate::state::MerkleTree;

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: MerkleTreeInstruction =
        MerkleTreeInstruction::try_from_slice(instruction_data)?;
    match instruction {
        MerkleTreeInstruction::Create() => {
            msg!("Instruction: Create");
            create(program_id, accounts)
        },
        MerkleTreeInstruction::InsertLeaf(insert_leaf_args) => {
            msg!("Instruction: InsertLeaf");
            insert_leaf(accounts, insert_leaf_args)
        }
    }
}

fn insert_leaf<'a>(accounts: &'a [AccountInfo<'a>], insert_leaf_args: InsertLeafArgs) -> ProgramResult {
    msg!("Mock insert leaf");
    let ctx = InsertLeafAccounts::context(accounts)?;
    let mut tree = MerkleTree::load(ctx.accounts.tree)?;
    tree.insert_leaf(insert_leaf_args.leaf)?;
    tree.save(ctx.accounts.tree)?;
    msg!("Leaf inserted. Root: {}", hex::encode(tree.nodes[0]));
    Ok(())
}

fn create<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    // Accounts.
    let ctx = CreateAccounts::context(accounts)?;
    let rent = Rent::get()?;

    // Guards.
    if *ctx.accounts.system_program.key != system_program::id() {
        return Err(MerkleTreeStorageError::InvalidSystemProgram.into());
    }

    // Fetch the space and minimum lamports required for rent exemption.
    let space: usize = MerkleTree::TREE_SIZE_BYTES;
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
            &crate::id(),
        ),
        &[
            ctx.accounts.payer.clone(),
            ctx.accounts.tree.clone(),
            ctx.accounts.system_program.clone(),
        ],
        &[&[b"tree", ctx.accounts.payer.key.as_ref(), &[bump]]],
    )?;

    let tree = MerkleTree {
        nodes: vec![[0; 32]; MerkleTree::TREE_SIZE],
        next_leaf_index: 0,
    };

    tree.save(ctx.accounts.tree)?;

    msg!("Merkle tree PDA initialized");
    Ok(())
}
