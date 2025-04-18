use num_derive::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
pub enum MerkleTreeStorageError {
    /// 0 - Invalid System Program
    #[error("Invalid System Program")]
    InvalidSystemProgram = 0,
    /// 1 - Error deserializing account
    #[error("Error deserializing account")]
    DeserializationError = 1,
    /// 2 - Error serializing account
    #[error("Error serializing account")]
    SerializationError = 2,
    /// 3 - Tree overflow
    #[error("Tree overflow")]
    TreeOverflow = 3,
    /// 4 - Invalid PDA
    #[error("Invalid PDA")]
    InvalidPDA = 4,
    /// 5 - Payer must be a signer
    #[error("Payer must be a signer")]
    PayerMustBeSigner = 5,
}

impl PrintProgramError for MerkleTreeStorageError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<MerkleTreeStorageError> for ProgramError {
    fn from(e: MerkleTreeStorageError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for MerkleTreeStorageError {
    fn type_of() -> &'static str {
        "Merkle Tree Storage Error"
    }
}
