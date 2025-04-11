use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;

use crate::error::MerkleTreeStorageError;

const MAX_DEPTH: usize = 3; // Tree with 8 leaves max
const TREE_SIZE: usize = (1 << (MAX_DEPTH + 1)) - 1; // 15 nodes

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct MerkleTree {
    pub nodes: Vec<[u8; 32]>,
    pub next_leaf_index: u8, // index of the next free leaf
}

impl MerkleTree {
    pub const TREE_SIZE_BYTES: usize = TREE_SIZE * 32 + 1; // bytes for all nodes + 1 byte for next_leaf_index

    pub fn load(account: &AccountInfo) -> Result<Self, ProgramError> {
        let mut bytes: &[u8] = &(*account.data).borrow();
        MerkleTree::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            MerkleTreeStorageError::DeserializationError.into()
        })
    }

    pub fn save(&self, account: &AccountInfo) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[..], self).map_err(|error| {
            msg!("Error: {}", error);
            MerkleTreeStorageError::SerializationError.into()
        })
    }
}
