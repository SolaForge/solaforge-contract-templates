//! Error types

use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the Token program
#[derive(Error, Debug, Copy, Clone)]
pub enum TokenError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    /// Not rent exempt
    #[error("Not rent exempt")]
    NotRentExempt,
    
    /// Expected mint
    #[error("Expected a mint account")]
    ExpectedMint,
    
    /// Expected token account
    #[error("Expected a token account")]
    ExpectedTokenAccount,
    
    /// Owner mismatch
    #[error("Account owner mismatch")]
    OwnerMismatch,
    
    /// Insufficient funds
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    /// Unauthorized operation
    #[error("Unauthorized operation")]
    Unauthorized,
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
