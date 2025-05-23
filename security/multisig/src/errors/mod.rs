//! Error types

use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the Multisig program
#[derive(Error, Debug, Copy, Clone)]
pub enum MultisigError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    /// Not rent exempt
    #[error("Not rent exempt")]
    NotRentExempt,
    
    /// Account already in use
    #[error("Account already in use")]
    AccountAlreadyInUse,
    
    /// Multisig account is not initialized
    #[error("Multisig account is not initialized")]
    UninitializedAccount,
    
    /// Invalid owner
    #[error("Invalid owner")]
    InvalidOwner,
    
    /// Invalid number of signers
    #[error("Invalid number of signers")]
    InvalidNumberOfSigners,
    
    /// Not enough signers
    #[error("Not enough signers")]
    NotEnoughSigners,
    
    /// Transaction already executed
    #[error("Transaction already executed")]
    AlreadyExecuted,
    
    /// Owners must be unique
    #[error("Owners must be unique")]
    DuplicateOwner,
    
    /// Owner not found
    #[error("Owner not found")]
    OwnerNotFound,
    
    /// Owner already signed
    #[error("Owner already signed")]
    OwnerAlreadySigned,
    
    /// Transaction index out of bounds
    #[error("Transaction index out of bounds")]
    TransactionIndexOutOfBounds,
    
    /// Transaction not ready for execution
    #[error("Transaction not ready for execution")]
    TransactionNotReady,
    
    /// Authority mismatch
    #[error("Authority mismatch")]
    AuthorityMismatch,
    
    /// Invalid transaction data
    #[error("Invalid transaction data")]
    InvalidTransactionData,
    
    /// Numerical overflow
    #[error("Numerical overflow")]
    NumericalOverflow,
}

impl From<MultisigError> for ProgramError {
    fn from(e: MultisigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
