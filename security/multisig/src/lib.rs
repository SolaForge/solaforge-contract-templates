//! Multisig Security program
//!
//! This program provides a flexible multisignature security implementation
//! for Solana programs. It allows multiple signers to approve transactions
//! before they are executed, providing enhanced security for critical operations.

pub mod instructions;
pub mod state;
pub mod utils;
pub mod errors;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("MuLti51gkEJZAQYYcE5Gfx2qC4nC6YtQJLyLBzf5vPGW");

/// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    instruction_data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    instructions::processor::process_instruction(program_id, accounts, instruction_data)
}
