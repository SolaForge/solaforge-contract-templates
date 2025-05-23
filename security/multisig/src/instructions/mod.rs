//! Instruction types

pub mod processor;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

/// Instructions supported by the Multisig program
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum MultisigInstruction {
    /// Creates a new multisig account
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The funding account (pays for account creation)
    /// 1. `[writable]` The multisig account to create
    /// 2. `[]` The system program
    /// 3. `[]` The rent sysvar
    ///
    CreateMultisig {
        /// Number of signers required to approve a transaction
        threshold: u8,
        /// List of owner public keys
        owners: Vec<Pubkey>,
    },

    /// Creates a new transaction
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The owner that creates the transaction (must be one of the multisig owners)
    /// 1. `[writable]` The multisig account
    /// 2. `[writable]` The transaction account to create
    /// 3. `[]` The system program
    /// 4. `[]` The rent sysvar
    ///
    CreateTransaction {
        /// Serialized transaction data
        /// Format: program_id (32 bytes) || accounts_len (1 byte) || accounts || data_len (2 bytes) || data
        /// where accounts is a list of AccountMeta serialized as:
        /// pubkey (32 bytes) || is_signer (1 byte) || is_writable (1 byte)
        transaction_data: Vec<u8>,
    },

    /// Approves a transaction
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The owner that approves the transaction (must be one of the multisig owners)
    /// 1. `[writable]` The multisig account
    /// 2. `[writable]` The transaction account
    ///
    ApproveTransaction,

    /// Executes a transaction
    ///
    /// Accounts expected:
    /// 0. `[signer]` The owner that executes the transaction (must be one of the multisig owners)
    /// 1. `[writable]` The multisig account
    /// 2. `[writable]` The transaction account
    /// 3+. `[writable, signer]` Remaining accounts depend on the transaction being executed
    ///
    ExecuteTransaction,

    /// Removes a transaction
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The owner that removes the transaction (must be one of the multisig owners or creator)
    /// 1. `[writable]` The multisig account
    /// 2. `[writable]` The transaction account
    ///
    RemoveTransaction,

    /// Change the owners or threshold
    ///
    /// Accounts expected:
    /// 0. `[writable]` The multisig account
    /// 1. `[writable]` The transaction account - must be a previously approved transaction
    ///    that contains this ChangeOwners instruction
    ///
    ChangeOwners {
        /// New threshold
        threshold: u8,
        /// New list of owner public keys
        owners: Vec<Pubkey>,
    },
}

/// Creates a CreateMultisig instruction
pub fn create_multisig(
    program_id: &Pubkey,
    funder: &Pubkey,
    multisig_account: &Pubkey,
    owners: Vec<Pubkey>,
    threshold: u8,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*funder, true),
        AccountMeta::new(*multisig_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    let data = MultisigInstruction::CreateMultisig { threshold, owners };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates a CreateTransaction instruction
pub fn create_transaction(
    program_id: &Pubkey,
    owner: &Pubkey,
    multisig_account: &Pubkey,
    transaction_account: &Pubkey,
    transaction_data: Vec<u8>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*owner, true),
        AccountMeta::new(*multisig_account, false),
        AccountMeta::new(*transaction_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    let data = MultisigInstruction::CreateTransaction { transaction_data };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an ApproveTransaction instruction
pub fn approve_transaction(
    program_id: &Pubkey,
    owner: &Pubkey,
    multisig_account: &Pubkey,
    transaction_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*owner, true),
        AccountMeta::new(*multisig_account, false),
        AccountMeta::new(*transaction_account, false),
    ];

    let data = MultisigInstruction::ApproveTransaction;

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an ExecuteTransaction instruction
pub fn execute_transaction(
    program_id: &Pubkey,
    owner: &Pubkey,
    multisig_account: &Pubkey,
    transaction_account: &Pubkey,
    other_accounts: Vec<AccountMeta>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new(*multisig_account, false),
        AccountMeta::new(*transaction_account, false),
    ];
    accounts.extend(other_accounts);

    let data = MultisigInstruction::ExecuteTransaction;

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates a RemoveTransaction instruction
pub fn remove_transaction(
    program_id: &Pubkey,
    owner: &Pubkey,
    multisig_account: &Pubkey,
    transaction_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*owner, true),
        AccountMeta::new(*multisig_account, false),
        AccountMeta::new(*transaction_account, false),
    ];

    let data = MultisigInstruction::RemoveTransaction;

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates a ChangeOwners instruction
pub fn change_owners(
    program_id: &Pubkey,
    multisig_account: &Pubkey,
    transaction_account: &Pubkey,
    owners: Vec<Pubkey>,
    threshold: u8,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*multisig_account, false),
        AccountMeta::new(*transaction_account, false),
    ];

    let data = MultisigInstruction::ChangeOwners { threshold, owners };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}
