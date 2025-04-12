use sha2::{Digest, Sha256};

/// A hasher compatible with solana_program::hash::hashv (SHA256)
pub struct SolanaHasher;

impl SolanaHasher {
    fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

pub struct OffchainMerkleTree {
    pub nodes: Vec<[u8; 32]>,
    pub next_leaf_index: u8, // index of the next free leaf
}

const MAX_DEPTH: usize = 3; // Tree with 8 leaves max

impl OffchainMerkleTree {
    pub const TREE_SIZE: usize = (1 << (MAX_DEPTH + 1)) - 1; // 15 nodes
    pub const TREE_SIZE_BYTES: usize = 8 + Self::TREE_SIZE * 32 + 1; // 8 bytes for vec length + bytes for all nodes + 1 byte for next_leaf_index

    pub fn insert_leaf(&mut self, leaf: [u8; 32]) -> Result<(), ()> {
        let leaf_pos = (1 << MAX_DEPTH) - 1 + self.next_leaf_index as usize;
        if leaf_pos >= Self::TREE_SIZE {
            return Err(());
        }

        self.nodes[leaf_pos] = leaf;
        let mut current = leaf_pos;

        while current > 0 {
            let parent = (current - 1) / 2;
            let left = self.nodes[2 * parent + 1];
            let right = self.nodes[2 * parent + 2];

            //concatenate left and right
            let mut concatenated = [left, right].concat();

            self.nodes[parent] = SolanaHasher::hash(&concatenated);

            current = parent;
        }

        self.next_leaf_index += 1;
        Ok(())
    }
}
