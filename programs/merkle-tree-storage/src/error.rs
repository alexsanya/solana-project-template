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
    InvalidSystemProgram,
    /// 1 - Error deserializing account
    #[error("Error deserializing account")]
    DeserializationError,
    /// 2 - Error serializing account
    #[error("Error serializing account")]
    SerializationError,
    /// 3 - Tree overflow
    #[error("Tree overflow")]
    TreeOverflow,
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
