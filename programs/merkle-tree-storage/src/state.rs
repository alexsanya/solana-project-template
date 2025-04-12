use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::hash::hashv;
use solana_program::msg;
use solana_program::program_error::ProgramError;

use crate::error::MerkleTreeStorageError;

const MAX_DEPTH: usize = 3; // Tree with 8 leaves max

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct MerkleTree {
    pub nodes: Vec<[u8; 32]>,
    pub next_leaf_index: u8, // index of the next free leaf
}

impl MerkleTree {
    pub const TREE_SIZE: usize = (1 << (MAX_DEPTH + 1)) - 1; // 15 nodes
    pub const TREE_SIZE_BYTES: usize = 8 + Self::TREE_SIZE * 32 + 1; // 8 bytes for vec length + bytes for all nodes + 1 byte for next_leaf_index

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

    pub fn insert_leaf(&mut self, leaf: [u8; 32]) -> Result<(), MerkleTreeStorageError> {
        let leaf_pos = (1 << MAX_DEPTH) - 1 + self.next_leaf_index as usize;
        if leaf_pos >= Self::TREE_SIZE {
            msg!("Tree is full");
            return Err(MerkleTreeStorageError::TreeOverflow);
        }

        self.nodes[leaf_pos] = leaf;
        let mut current = leaf_pos;

        while current > 0 {
            let parent = (current - 1) / 2;
            let left = self.nodes[2 * parent + 1];
            let right = self.nodes[2 * parent + 2];

            self.nodes[parent] = hashv(&[left.as_slice(), right.as_slice()]).to_bytes();

            current = parent;
        }

        self.next_leaf_index += 1;
        Ok(())
    }
}
