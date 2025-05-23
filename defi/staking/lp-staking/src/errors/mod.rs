//! Error types

use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the lp-token-staking program
#[derive(Error, Debug, Copy, Clone)]
pub enum TemplateError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    /// Not rent exempt
    #[error("Not rent exempt")]
    NotRentExempt,
    
    /// Expected amount mismatch
    #[error("Expected amount mismatch")]
    ExpectedAmountMismatch,
    
    /// Invalid authority
    #[error("Invalid authority")]
    InvalidAuthority,
    
    /// Math operation overflow
    #[error("Math operation overflow")]
    MathOverflow,
}

impl From<TemplateError> for ProgramError {
    fn from(e: TemplateError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
