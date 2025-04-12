use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::hash::hashv;
use solana_program::msg;
use solana_program::program_error::ProgramError;

use crate::error::MerkleTreeStorageError;

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct MerkleTree {
    pub nodes: Vec<[u8; 32]>,
    pub max_depth: u8,
    pub next_leaf_index: u8, // index of the next free leaf
}

impl MerkleTree {

    pub fn get_tree_size(max_depth: u8) -> usize {
        return (1 << (max_depth + 1)) - 1;
    }

    pub fn get_tree_size_bytes(max_depth: u8) -> usize {
        let tree_size = Self::get_tree_size(max_depth);
        return 8 + tree_size * 32 + 2; // 8 bytes for vec length + bytes for all nodes + 1 byte for next_leaf_index + 1 byte for max_depth
    }

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
        let leaf_pos = (1 << self.max_depth) - 1 + self.next_leaf_index as usize;
        if leaf_pos >= Self::get_tree_size(self.max_depth) {
            msg!("Tree is full");
            return Err(MerkleTreeStorageError::TreeOverflow);
        }

        self.nodes[leaf_pos] = leaf;
        let mut current = leaf_pos;

        while current > 0 {
            let parent = (current - 1) / 2;
            let left = self.nodes[2 * parent + 1];
            let right = self.nodes[2 * parent + 2];

            self.nodes[parent] = hashv(&[&left, &right]).to_bytes();

            current = parent;
        }

        self.next_leaf_index += 1;
        Ok(())
    }
}
