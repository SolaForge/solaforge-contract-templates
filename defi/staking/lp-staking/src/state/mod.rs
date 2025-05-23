//! State objects for template account

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Template account data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TemplateAccount {
    /// The account authority (owner)
    pub authority: Pubkey,
    
    /// Example value 1
    pub value1: u64,
    
    /// Example value 2
    pub value2: u8,
    
    /// Is the account initialized
    pub is_initialized: bool,
}
