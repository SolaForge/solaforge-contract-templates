//! Utils for multisig security

use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::errors::MultisigError;

/// Assert that an account is owned by a specific program
pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<(), ProgramError> {
    if account.owner != owner {
        Err(MultisigError::InvalidOwner.into())
    } else {
        Ok(())
    }
}

/// Calculate number of approvals needed for a multisig
pub fn calculate_approvals_needed(total_owners: usize, threshold: u8) -> usize {
    threshold as usize
}

/// Check if a transaction has enough approvals
pub fn has_enough_approvals(
    signers: &[bool],
    threshold: u8,
) -> bool {
    let approval_count = signers.iter().filter(|&approved| *approved).count();
    approval_count >= threshold as usize
}
