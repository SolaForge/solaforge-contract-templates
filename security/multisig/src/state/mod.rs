//! State objects for multisig security

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Multisig account data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MultisigAccount {
    /// Is the account initialized
    pub is_initialized: bool,
    
    /// Threshold of signatures required
    pub threshold: u8,
    
    /// List of authorized signers
    pub owners: Vec<Pubkey>,
    
    /// Number of transactions created
    pub transaction_count: u64,
}

impl MultisigAccount {
    /// Get the packed length of the account data
    pub fn get_packed_len(&self) -> usize {
        // is_initialized (1) + threshold (1) + owners length (4) + owners (32 * len) + transaction_count (8) + padding
        1 + 1 + 4 + (self.owners.len() * 32) + 8 + 32
    }
}

/// Status of a multisig transaction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    /// Transaction is active and can be signed
    Active,
    
    /// Transaction has been executed
    Executed,
    
    /// Transaction has been removed
    Removed,
}

/// Transaction account data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Transaction {
    /// The multisig account this transaction belongs to
    pub multisig: Pubkey,
    
    /// Current status
    pub status: TransactionStatus,
    
    /// The transaction data to execute
    pub transaction_data: Vec<u8>,
    
    /// Bit flags for signatures present
    pub signers: Vec<bool>,
    
    /// Creator of the transaction
    pub creator: Pubkey,
    
    /// When the transaction was executed
    pub executed_at: u64,
}

impl Transaction {
    /// Get the packed length of the transaction data
    pub fn get_packed_len(&self) -> usize {
        // multisig (32) + status (1) + transaction_data length (4) + transaction_data (len) + 
        // signers length (4) + signers (1 * len) + creator (32) + executed_at (8) + padding
        32 + 1 + 4 + self.transaction_data.len() + 4 + self.signers.len() + 32 + 8 + 32
    }
}
