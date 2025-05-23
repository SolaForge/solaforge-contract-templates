//! Instruction types

pub mod processor;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

/// Instructions supported by the perpetual-futures program
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum TemplateInstruction {
    /// Initialize a new instance
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The authority account
    /// 1. `[writable]` The account to initialize
    /// 2. `[]` The system program
    /// 3. `[]` The rent sysvar
    ///
    Initialize {
        /// Custom parameter 1
        param1: u64,
        /// Custom parameter 2
        param2: u8,
    },

    /// Perform operation 1
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The authority account
    /// 1. `[writable]` The account to modify
    ///
    Operation1 {
        /// Operation amount
        amount: u64,
    },

    /// Perform operation 2
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The authority account
    /// 1. `[writable]` The account to modify
    ///
    Operation2,
}

/// Creates an Initialize instruction
pub fn initialize(
    program_id: &Pubkey,
    authority: &Pubkey,
    account: &Pubkey,
    param1: u64,
    param2: u8,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    let data = TemplateInstruction::Initialize { param1, param2 };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an Operation1 instruction
pub fn operation1(
    program_id: &Pubkey,
    authority: &Pubkey,
    account: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*account, false),
    ];

    let data = TemplateInstruction::Operation1 { amount };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an Operation2 instruction
pub fn operation2(
    program_id: &Pubkey,
    authority: &Pubkey,
    account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*account, false),
    ];

    let data = TemplateInstruction::Operation2;

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}
