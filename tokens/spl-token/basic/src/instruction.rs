//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

/// Instructions supported by the Token program
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum TokenInstruction {
    /// Initialize a new token
    ///
    /// Accounts expected:
    /// 0. `[writable]` The token mint account
    /// 1. `[]` The rent sysvar
    /// 2. `[]` The token program ID
    /// 3. `[signer]` The mint authority
    ///
    /// Data: name, symbol, decimals
    InitializeMint {
        /// Token name
        name: String,
        /// Token symbol
        symbol: String,
        /// Number of decimals in token units
        decimals: u8,
    },

    /// Initialize a token account
    ///
    /// Accounts expected:
    /// 0. `[writable]` The token account to initialize
    /// 1. `[]` The token mint
    /// 2. `[]` The rent sysvar
    /// 3. `[]` The token program ID
    /// 4. `[signer]` The owner of the token account
    ///
    InitializeAccount,

    /// Mint new tokens to an account
    ///
    /// Accounts expected:
    /// 0. `[writable]` The mint account
    /// 1. `[writable]` The destination account
    /// 2. `[signer]` The mint authority
    /// 3. `[]` The token program ID
    ///
    /// Data: amount
    MintTo {
        /// Amount of tokens to mint
        amount: u64,
    },

    /// Transfer tokens
    ///
    /// Accounts expected:
    /// 0. `[writable]` The source account
    /// 1. `[writable]` The destination account
    /// 2. `[signer]` The owner of the source account
    /// 3. `[]` The token program ID
    ///
    /// Data: amount
    Transfer {
        /// Amount of tokens to transfer
        amount: u64,
    },
}

/// Create InitializeMint instruction
pub fn initialize_mint(
    program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    name: String,
    symbol: String,
    decimals: u8,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*mint_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(*mint_authority_pubkey, true),
    ];
    
    let data = TokenInstruction::InitializeMint {
        name,
        symbol,
        decimals,
    };
    
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Create InitializeAccount instruction
pub fn initialize_account(
    program_id: &Pubkey,
    account_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*account_pubkey, false),
        AccountMeta::new_readonly(*mint_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(*owner_pubkey, true),
    ];
    
    let data = TokenInstruction::InitializeAccount;
    
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Create MintTo instruction
pub fn mint_to(
    program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*mint_pubkey, false),
        AccountMeta::new(*destination_pubkey, false),
        AccountMeta::new_readonly(*mint_authority_pubkey, true),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    
    let data = TokenInstruction::MintTo {
        amount,
    };
    
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Create Transfer instruction
pub fn transfer(
    program_id: &Pubkey,
    source_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*source_pubkey, false),
        AccountMeta::new(*destination_pubkey, false),
        AccountMeta::new_readonly(*owner_pubkey, true),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    
    let data = TokenInstruction::Transfer {
        amount,
    };
    
    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}
