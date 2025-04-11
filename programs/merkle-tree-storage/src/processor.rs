use borsh::BorshDeserialize;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke, pubkey::Pubkey,
    rent::Rent, system_instruction, system_program, sysvar::Sysvar,
};

use crate::error::MerkleTreeStorageError;
use crate::instruction::accounts::CreateAccounts;
use crate::instruction::CreatePDAinstruction;
use crate::state::MerkleTree;

pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: CreatePDAinstruction =
        CreatePDAinstruction::try_from_slice(instruction_data)?;
    match instruction {
        CreatePDAinstruction::Create() => {
            msg!("Instruction: Create");
            create(program_id, accounts)
        }
    }
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

    msg!("Merkle tree PDA initialized");
    Ok(())
}
