//! Error types

use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Errors that may be returned by the NFT Marketplace program
#[derive(Error, Debug, Copy, Clone)]
pub enum MarketplaceError {
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    /// Not rent exempt
    #[error("Not rent exempt")]
    NotRentExempt,
    
    /// Expected amount mismatch
    #[error("Expected amount mismatch")]
    ExpectedAmountMismatch,
    
    /// Listing already canceled
    #[error("Listing already canceled")]
    ListingAlreadyCanceled,
    
    /// Listing already executed
    #[error("Listing already executed")]
    ListingAlreadyExecuted,
    
    /// Listing not active
    #[error("Listing not active")]
    ListingNotActive,
    
    /// Invalid listing price
    #[error("Invalid listing price")]
    InvalidListingPrice,
    
    /// Invalid metadata
    #[error("Invalid metadata")]
    InvalidMetadata,
    
    /// Not NFT owner
    #[error("Not NFT owner")]
    NotNFTOwner,
    
    /// NFT account mismatch
    #[error("NFT account mismatch")]
    NFTAccountMismatch,
    
    /// Authority mismatch
    #[error("Authority mismatch")]
    AuthorityMismatch,
    
    /// Invalid treasury account
    #[error("Invalid treasury account")]
    InvalidTreasuryAccount,
    
    /// Numerical overflow
    #[error("Numerical overflow")]
    NumericalOverflow,
}

impl From<MarketplaceError> for ProgramError {
    fn from(e: MarketplaceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
