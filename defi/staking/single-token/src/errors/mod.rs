//! Error types

use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the Staking program
#[derive(Error, Debug, Copy, Clone)]
pub enum StakingError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    /// Not rent exempt
    #[error("Not rent exempt")]
    NotRentExempt,
    
    /// Expected amount mismatch
    #[error("Expected amount mismatch")]
    ExpectedAmountMismatch,
    
    /// Stake pool already initialized
    #[error("Stake pool already initialized")]
    PoolAlreadyInitialized,
    
    /// Stake pool not initialized
    #[error("Stake pool not initialized")]
    PoolNotInitialized,
    
    /// Invalid token program
    #[error("Invalid token program")]
    InvalidTokenProgram,
    
    /// Invalid stake account
    #[error("Invalid stake account")]
    InvalidStakeAccount,
    
    /// Invalid token account
    #[error("Invalid token account")]
    InvalidTokenAccount,
    
    /// Unauthorized access
    #[error("Unauthorized access")]
    Unauthorized,
    
    /// Withdraw too early
    #[error("Withdrawal before lock period ends")]
    WithdrawTooEarly,
    
    /// Insufficient stake
    #[error("Insufficient stake amount")]
    InsufficientStake,
    
    /// Insufficient funds
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    /// Invalid stake duration
    #[error("Invalid stake duration")]
    InvalidStakeDuration,
    
    /// Invalid reward rate
    #[error("Invalid reward rate")]
    InvalidRewardRate,
    
    /// Numerical overflow
    #[error("Numerical overflow")]
    NumericalOverflow,
    
    /// Invalid stake pool
    #[error("Invalid stake pool")]
    InvalidStakePool,
}

impl From<StakingError> for ProgramError {
    fn from(e: StakingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
