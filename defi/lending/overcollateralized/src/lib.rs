//! Overcollateralized lending protocol
//!
//! This program provides [TEMPLATE FUNCTIONALITY].

pub mod instructions;
pub mod state;
pub mod utils;
pub mod errors;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("overcollateralized-lending111111111111111111111111111111");

/// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    instruction_data: &[u8],
) -> solana_program::entrypoint::ProgramResult {
    instructions::processor::process_instruction(program_id, accounts, instruction_data)
}
