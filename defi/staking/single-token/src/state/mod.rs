//! State objects for staking

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Staking pool data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct StakePool {
    /// Authority that can update the pool
    pub authority: Pubkey,
    
    /// Token mint for the staking token
    pub token_mint: Pubkey,
    
    /// Pool token account to hold staked tokens
    pub pool_token_account: Pubkey,
    
    /// Pool reward account to distribute rewards from
    pub pool_reward_account: Pubkey,
    
    /// Reward rate in basis points per day (e.g., 100 = 1%)
    pub reward_rate: u64,
    
    /// Minimum stake duration in seconds
    pub min_stake_duration: u64,
    
    /// Early withdrawal penalty in basis points (e.g., 500 = 5%)
    pub early_withdrawal_penalty: u16,
    
    /// Total tokens staked in the pool
    pub total_staked: u64,
    
    /// Total number of stakers
    pub total_stakers: u64,
    
    /// Total rewards distributed so far
    pub total_rewards_distributed: u64,
    
    /// Available reward funds
    pub reward_funds_available: u64,
    
    /// Last time the pool was updated
    pub last_updated_timestamp: u64,
}

impl StakePool {
    /// Get the size of StakePool struct
    pub fn get_size() -> usize {
        // Pubkey (32 bytes) * 4 + reward_rate (8 bytes) + min_stake_duration (8 bytes) +
        // early_withdrawal_penalty (2 bytes) + total_staked (8 bytes) + total_stakers (8 bytes) +
        // total_rewards_distributed (8 bytes) + reward_funds_available (8 bytes) +
        // last_updated_timestamp (8 bytes) + some padding
        32 * 4 + 8 + 8 + 2 + 8 + 8 + 8 + 8 + 8 + 8
    }
}

/// User stake data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserStake {
    /// Owner of the stake
    pub owner: Pubkey,
    
    /// Staking pool this stake belongs to
    pub pool: Pubkey,
    
    /// Amount of tokens staked
    pub stake_amount: u64,
    
    /// Total rewards claimed so far
    pub rewards_claimed: u64,
    
    /// When the stake was created
    pub stake_timestamp: u64,
    
    /// When the stake can be withdrawn without penalty
    pub unlock_timestamp: u64,
    
    /// Last time rewards were claimed
    pub last_claim_timestamp: u64,
}

impl UserStake {
    /// Get the size of UserStake struct
    pub fn get_size() -> usize {
        // Pubkey (32 bytes) * 2 + stake_amount (8 bytes) + rewards_claimed (8 bytes) +
        // stake_timestamp (8 bytes) + unlock_timestamp (8 bytes) + 
        // last_claim_timestamp (8 bytes) + some padding
        32 * 2 + 8 + 8 + 8 + 8 + 8 + 8
    }
}
