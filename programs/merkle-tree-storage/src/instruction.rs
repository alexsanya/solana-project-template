use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};



#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, ShankContext, ShankInstruction)]
#[rustfmt::skip]
pub enum CreatePDAinstruction {
    /// Create Tree storage account
    #[account(0, writable, signer, name="payer", desc = "The account paying for the storage fees")]
    #[account(1, writable, name="tree", desc = "The address of the new account")]
    #[account(2, name="system_program", desc="The system program")]
    #[account(3, name="sysvar_rent", desc="Sysvar rent account")]
    Create(),
}
