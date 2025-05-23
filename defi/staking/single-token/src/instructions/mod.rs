//! Instruction types

pub mod processor;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

/// Instructions supported by the Staking program
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum StakingInstruction {
    /// Initialize a new staking pool
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The authority that will control the pool
    /// 1. `[writable]` The stake pool account to initialize
    /// 2. `[]` The SPL token mint for the staking token
    /// 3. `[writable]` The token account that will hold staked tokens
    /// 4. `[writable]` The token account that will hold reward tokens
    /// 5. `[]` The token program
    /// 6. `[]` The system program
    /// 7. `[]` The rent sysvar
    ///
    InitializePool {
        /// Base reward rate (tokens per token per second in basis points)
        reward_rate: u64,
        /// Minimum staking duration in seconds
        min_stake_duration: u64,
        /// Early withdrawal penalty percentage in basis points
        early_withdrawal_penalty: u16,
    },

    /// Stake tokens in the pool
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The user staking tokens
    /// 1. `[writable]` The stake pool account
    /// 2. `[writable]` The pool's token account
    /// 3. `[writable]` The user's stake account to create
    /// 4. `[writable]` The user's token account to withdraw from
    /// 5. `[]` The token program
    /// 6. `[]` The system program
    /// 7. `[]` The rent sysvar
    ///
    Stake {
        /// Amount of tokens to stake
        amount: u64,
        /// Custom lock duration in seconds (0 = use pool minimum)
        lock_duration: u64,
    },

    /// Unstake tokens from the pool
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The user unstaking tokens
    /// 1. `[writable]` The stake pool account
    /// 2. `[writable]` The pool's token account
    /// 3. `[writable]` The user's stake account
    /// 4. `[writable]` The user's token account to receive principal
    /// 5. `[writable]` The pool's reward token account
    /// 6. `[writable]` The user's token account to receive rewards
    /// 7. `[]` The token program
    ///
    Unstake {
        /// Amount of tokens to unstake (0 = all)
        amount: u64,
    },

    /// Claim rewards without unstaking
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The user claiming rewards
    /// 1. `[writable]` The stake pool account
    /// 2. `[writable]` The user's stake account
    /// 3. `[writable]` The pool's reward token account
    /// 4. `[writable]` The user's token account to receive rewards
    /// 5. `[]` The token program
    ///
    ClaimRewards,

    /// Update pool parameters
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The pool authority
    /// 1. `[writable]` The stake pool account
    ///
    UpdatePool {
        /// New reward rate
        reward_rate: u64,
        /// New minimum stake duration
        min_stake_duration: u64,
        /// New early withdrawal penalty
        early_withdrawal_penalty: u16,
    },

    /// Fund the reward pool
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The funder account (can be authority or anyone)
    /// 1. `[writable]` The stake pool account
    /// 2. `[writable]` The funder's token account
    /// 3. `[writable]` The pool's reward token account
    /// 4. `[]` The token program
    ///
    FundRewards {
        /// Amount of reward tokens to add
        amount: u64,
    },
}

/// Creates an instruction to initialize a staking pool
pub fn initialize_pool(
    program_id: &Pubkey,
    authority: &Pubkey,
    stake_pool: &Pubkey,
    token_mint: &Pubkey,
    pool_token_account: &Pubkey,
    pool_reward_account: &Pubkey,
    reward_rate: u64,
    min_stake_duration: u64,
    early_withdrawal_penalty: u16,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new_readonly(*token_mint, false),
        AccountMeta::new(*pool_token_account, false),
        AccountMeta::new(*pool_reward_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    let data = StakingInstruction::InitializePool {
        reward_rate,
        min_stake_duration,
        early_withdrawal_penalty,
    };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an instruction to stake tokens
pub fn stake(
    program_id: &Pubkey,
    user: &Pubkey,
    stake_pool: &Pubkey,
    pool_token_account: &Pubkey,
    user_stake_account: &Pubkey,
    user_token_account: &Pubkey,
    amount: u64,
    lock_duration: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*user, true),
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new(*pool_token_account, false),
        AccountMeta::new(*user_stake_account, false),
        AccountMeta::new(*user_token_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    let data = StakingInstruction::Stake {
        amount,
        lock_duration,
    };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an instruction to unstake tokens
pub fn unstake(
    program_id: &Pubkey,
    user: &Pubkey,
    stake_pool: &Pubkey,
    pool_token_account: &Pubkey,
    user_stake_account: &Pubkey,
    user_token_account: &Pubkey,
    pool_reward_account: &Pubkey,
    user_reward_account: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*user, true),
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new(*pool_token_account, false),
        AccountMeta::new(*user_stake_account, false),
        AccountMeta::new(*user_token_account, false),
        AccountMeta::new(*pool_reward_account, false),
        AccountMeta::new(*user_reward_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let data = StakingInstruction::Unstake { amount };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an instruction to claim rewards
pub fn claim_rewards(
    program_id: &Pubkey,
    user: &Pubkey,
    stake_pool: &Pubkey,
    user_stake_account: &Pubkey,
    pool_reward_account: &Pubkey,
    user_reward_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*user, true),
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new(*user_stake_account, false),
        AccountMeta::new(*pool_reward_account, false),
        AccountMeta::new(*user_reward_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let data = StakingInstruction::ClaimRewards;

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an instruction to update pool parameters
pub fn update_pool(
    program_id: &Pubkey,
    authority: &Pubkey,
    stake_pool: &Pubkey,
    reward_rate: u64,
    min_stake_duration: u64,
    early_withdrawal_penalty: u16,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*stake_pool, false),
    ];

    let data = StakingInstruction::UpdatePool {
        reward_rate,
        min_stake_duration,
        early_withdrawal_penalty,
    };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an instruction to fund the reward pool
pub fn fund_rewards(
    program_id: &Pubkey,
    funder: &Pubkey,
    stake_pool: &Pubkey,
    funder_token_account: &Pubkey,
    pool_reward_account: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*funder, true),
        AccountMeta::new(*stake_pool, false),
        AccountMeta::new(*funder_token_account, false),
        AccountMeta::new(*pool_reward_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let data = StakingInstruction::FundRewards { amount };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}
