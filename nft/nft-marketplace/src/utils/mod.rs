//! Utils for NFT marketplace

use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::errors::MarketplaceError;

/// Assert that an account is owned by a specific program
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<(), ProgramError> {
    if account.owner != owner {
        Err(MarketplaceError::AuthorityMismatch.into())
    } else {
        Ok(())
    }
}

/// Calculate marketplace fee
pub fn calculate_fee(price: u64, fee_basis_points: u16) -> Result<u64, ProgramError> {
    price
        .checked_mul(fee_basis_points as u64)
        .ok_or(MarketplaceError::NumericalOverflow.into())?
        .checked_div(10000)
        .ok_or(MarketplaceError::NumericalOverflow.into())
}
