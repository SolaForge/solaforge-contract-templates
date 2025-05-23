//! Utility functions for the program

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::errors::TemplateError;

/// Checks that the account is owned by the expected program
pub fn check_account_owner(account_info: &AccountInfo, program_id: &Pubkey) -> ProgramResult {
    if account_info.owner != program_id {
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

/// Checks that the given account is a signer
pub fn check_signer(account_info: &AccountInfo) -> ProgramResult {
    if !account_info.is_signer {
        Err(ProgramError::MissingRequiredSignature)
    } else {
        Ok(())
    }
}

/// Safely performs a mathematical addition that errors on overflow
pub fn safe_addition(a: u64, b: u64) -> Result<u64, TemplateError> {
    a.checked_add(b).ok_or(TemplateError::MathOverflow)
}
