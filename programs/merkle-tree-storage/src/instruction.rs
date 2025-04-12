use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum MerkleTreeInstruction {
    /// Create Tree storage account
    #[account(0, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(1, writable, name="tree", desc = "The address of the new account")]
    #[account(2, name="system_program", desc="The system program")]
    #[account(3, name="sysvar_rent", desc="Sysvar rent account")]
    CreateTree(CreateTreeArgs),

    /// Insert Leaf
    #[account(0, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(1, writable, name="tree", desc = "The address of the new account")]
    InsertLeaf(InsertLeafArgs),
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct InsertLeafArgs {
    pub leaf: [u8; 32]
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CreateTreeArgs {
    pub max_depth: u8
}
