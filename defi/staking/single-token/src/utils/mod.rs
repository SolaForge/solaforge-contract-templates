//! Utils for staking

use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::errors::StakingError;

/// Assert that an account is owned by a specific program
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<(), ProgramError> {
    if account.owner != owner {
        Err(StakingError::Unauthorized.into())
    } else {
        Ok(())
    }
}

/// Calculate APY from reward rate
pub fn calculate_apy(reward_rate: u64) -> f64 {
    // Convert reward rate from basis points per day to percentage per year
    let daily_percentage = (reward_rate as f64) / 10000.0;
    let apy = (1.0 + daily_percentage).powf(365.0) - 1.0;
    apy * 100.0 // Return as percentage value
}
