//! State types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Token mint account data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Mint {
    /// Authority that can mint new tokens
    pub mint_authority: Option<Pubkey>,
    
    /// Total supply of tokens
    pub supply: u64,
    
    /// Number of decimal places
    pub decimals: u8,
    
    /// Is the mint initialized
    pub is_initialized: bool,
    
    /// Optional authority that can freeze token accounts
    pub freeze_authority: Option<Pubkey>,
    
    /// Token name
    pub name: String,
    
    /// Token symbol
    pub symbol: String,
}

/// Token account data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct TokenAccount {
    /// The mint this account is associated with
    pub mint: Pubkey,
    
    /// The owner of this account
    pub owner: Pubkey,
    
    /// The amount of tokens this account holds
    pub amount: u64,
    
    /// If `true`, this account's tokens are frozen
    pub is_frozen: bool,
    
    /// Is this account initialized
    pub is_initialized: bool,
}
