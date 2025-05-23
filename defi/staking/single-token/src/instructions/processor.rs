//! Program instruction processor

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    errors::StakingError,
    instructions::StakingInstruction,
    state::{StakePool, UserStake},
    utils::assert_owned_by,
};

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StakingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        StakingInstruction::InitializePool {
            reward_rate,
            min_stake_duration,
            early_withdrawal_penalty,
        } => {
            msg!("Instruction: Initialize Pool");
            process_initialize_pool(
                program_id,
                accounts,
                reward_rate,
                min_stake_duration,
                early_withdrawal_penalty,
            )
        }
        StakingInstruction::Stake {
            amount,
            lock_duration,
        } => {
            msg!("Instruction: Stake");
            process_stake(program_id, accounts, amount, lock_duration)
        }
        StakingInstruction::Unstake { amount } => {
            msg!("Instruction: Unstake");
            process_unstake(program_id, accounts, amount)
        }
        StakingInstruction::ClaimRewards => {
            msg!("Instruction: Claim Rewards");
            process_claim_rewards(program_id, accounts)
        }
        StakingInstruction::UpdatePool {
            reward_rate,
            min_stake_duration,
            early_withdrawal_penalty,
        } => {
            msg!("Instruction: Update Pool");
            process_update_pool(
                program_id,
                accounts,
                reward_rate,
                min_stake_duration,
                early_withdrawal_penalty,
            )
        }
        StakingInstruction::FundRewards { amount } => {
            msg!("Instruction: Fund Rewards");
            process_fund_rewards(program_id, accounts, amount)
        }
    }
}

/// Process InitializePool instruction
fn process_initialize_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    reward_rate: u64,
    min_stake_duration: u64,
    early_withdrawal_penalty: u16,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let authority_info = next_account_info(account_info_iter)?;
    let stake_pool_info = next_account_info(account_info_iter)?;
    let token_mint_info = next_account_info(account_info_iter)?;
    let pool_token_account_info = next_account_info(account_info_iter)?;
    let pool_reward_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    
    // Check the authority is a signer
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate token program
    if *token_program_info.key != spl_token::id() {
        return Err(StakingError::InvalidTokenProgram.into());
    }
    
    // Validate reward rate
    if reward_rate == 0 {
        return Err(StakingError::InvalidRewardRate.into());
    }
    
    // Validate min stake duration
    if min_stake_duration == 0 {
        return Err(StakingError::InvalidStakeDuration.into());
    }
    
    // Validate early withdrawal penalty (max 100%)
    if early_withdrawal_penalty > 10000 {
        return Err(StakingError::InvalidRewardRate.into());
    }
    
    // Verify token accounts
    let pool_token_account = spl_token::state::Account::unpack(&pool_token_account_info.data.borrow())?;
    if pool_token_account.mint != *token_mint_info.key {
        return Err(StakingError::InvalidTokenAccount.into());
    }
    
    let pool_reward_account = spl_token::state::Account::unpack(&pool_reward_account_info.data.borrow())?;
    if pool_reward_account.mint != *token_mint_info.key {
        return Err(StakingError::InvalidTokenAccount.into());
    }
    
    // Create stake pool account
    let rent = &Rent::from_account_info(rent_info)?;
    let stake_pool_size = StakePool::get_size();
    let stake_pool_lamports = rent.minimum_balance(stake_pool_size);
    
    invoke(
        &system_instruction::create_account(
            authority_info.key,
            stake_pool_info.key,
            stake_pool_lamports,
            stake_pool_size as u64,
            program_id,
        ),
        &[
            authority_info.clone(),
            stake_pool_info.clone(),
            system_program_info.clone(),
        ],
    )?;
    
    // Initialize stake pool
    let stake_pool = StakePool {
        authority: *authority_info.key,
        token_mint: *token_mint_info.key,
        pool_token_account: *pool_token_account_info.key,
        pool_reward_account: *pool_reward_account_info.key,
        reward_rate,
        min_stake_duration,
        early_withdrawal_penalty,
        total_staked: 0,
        total_stakers: 0,
        total_rewards_distributed: 0,
        reward_funds_available: 0,
        last_updated_timestamp: Clock::get()?.unix_timestamp as u64,
    };
    
    stake_pool.serialize(&mut *stake_pool_info.data.borrow_mut())?;
    
    Ok(())
}

/// Process Stake instruction
fn process_stake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    lock_duration: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let user_info = next_account_info(account_info_iter)?;
    let stake_pool_info = next_account_info(account_info_iter)?;
    let pool_token_account_info = next_account_info(account_info_iter)?;
    let user_stake_account_info = next_account_info(account_info_iter)?;
    let user_token_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    
    // Check the user is a signer
    if !user_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate stake pool account
    assert_owned_by(stake_pool_info, program_id)?;
    
    // Deserialize the stake pool
    let mut stake_pool = StakePool::try_from_slice(&stake_pool_info.data.borrow())?;
    
    // Validate token accounts
    if stake_pool.pool_token_account != *pool_token_account_info.key {
        return Err(StakingError::InvalidTokenAccount.into());
    }
    
    // Validate amount
    if amount == 0 {
        return Err(StakingError::InsufficientStake.into());
    }
    
    // Validate lock duration - either 0 (use default) or >= min stake duration
    if lock_duration != 0 && lock_duration < stake_pool.min_stake_duration {
        return Err(StakingError::InvalidStakeDuration.into());
    }
    
    // Create user stake account if it doesn't exist
    let rent = &Rent::from_account_info(rent_info)?;
    let user_stake_size = UserStake::get_size();
    let user_stake_lamports = rent.minimum_balance(user_stake_size);
    
    // Only create if it doesn't exist yet
    if user_stake_account_info.data_is_empty() {
        invoke(
            &system_instruction::create_account(
                user_info.key,
                user_stake_account_info.key,
                user_stake_lamports,
                user_stake_size as u64,
                program_id,
            ),
            &[
                user_info.clone(),
                user_stake_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
        
        // Update staker count
        stake_pool.total_stakers = stake_pool.total_stakers.checked_add(1).ok_or(StakingError::NumericalOverflow)?;
    }
    
    // Transfer tokens from user to pool
    invoke(
        &spl_token::instruction::transfer(
            token_program_info.key,
            user_token_account_info.key,
            pool_token_account_info.key,
            user_info.key,
            &[],
            amount,
        )?,
        &[
            user_token_account_info.clone(),
            pool_token_account_info.clone(),
            user_info.clone(),
            token_program_info.clone(),
        ],
    )?;
    
    // Initialize or update user stake
    let current_time = Clock::get()?.unix_timestamp as u64;
    let mut user_stake = if user_stake_account_info.data_is_empty() {
        UserStake {
            owner: *user_info.key,
            pool: *stake_pool_info.key,
            stake_amount: amount,
            rewards_claimed: 0,
            stake_timestamp: current_time,
            unlock_timestamp: current_time + if lock_duration > 0 { lock_duration } else { stake_pool.min_stake_duration },
            last_claim_timestamp: current_time,
        }
    } else {
        assert_owned_by(user_stake_account_info, program_id)?;
        let mut existing_stake = UserStake::try_from_slice(&user_stake_account_info.data.borrow())?;
        
        // Verify stake belongs to correct user
        if existing_stake.owner != *user_info.key {
            return Err(StakingError::Unauthorized.into());
        }
        
        // Verify stake is for this pool
        if existing_stake.pool != *stake_pool_info.key {
            return Err(StakingError::InvalidStakePool.into());
        }
        
        // Calculate pending rewards first (so they're not lost)
        let pending_rewards = calculate_rewards(&existing_stake, &stake_pool, current_time);
        
        // Add new stake
        existing_stake.stake_amount = existing_stake.stake_amount.checked_add(amount).ok_or(StakingError::NumericalOverflow)?;
        
        // If adding more tokens, extend lock period if new one is longer
        let new_unlock = current_time + if lock_duration > 0 { lock_duration } else { stake_pool.min_stake_duration };
        if new_unlock > existing_stake.unlock_timestamp {
            existing_stake.unlock_timestamp = new_unlock;
        }
        
        // Store pending rewards internally
        existing_stake.rewards_claimed = existing_stake.rewards_claimed.checked_add(pending_rewards).ok_or(StakingError::NumericalOverflow)?;
        existing_stake.last_claim_timestamp = current_time;
        
        existing_stake
    };
    
    // Save user stake
    user_stake.serialize(&mut *user_stake_account_info.data.borrow_mut())?;
    
    // Update stake pool total staked
    stake_pool.total_staked = stake_pool.total_staked.checked_add(amount).ok_or(StakingError::NumericalOverflow)?;
    stake_pool.last_updated_timestamp = current_time;
    stake_pool.serialize(&mut *stake_pool_info.data.borrow_mut())?;
    
    Ok(())
}

/// Process Unstake instruction
fn process_unstake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let user_info = next_account_info(account_info_iter)?;
    let stake_pool_info = next_account_info(account_info_iter)?;
    let pool_token_account_info = next_account_info(account_info_iter)?;
    let user_stake_account_info = next_account_info(account_info_iter)?;
    let user_token_account_info = next_account_info(account_info_iter)?;
    let pool_reward_account_info = next_account_info(account_info_iter)?;
    let user_reward_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    
    // Check the user is a signer
    if !user_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate stake pool account
    assert_owned_by(stake_pool_info, program_id)?;
    
    // Validate user stake account
    assert_owned_by(user_stake_account_info, program_id)?;
    
    // Deserialize the stake pool and user stake
    let mut stake_pool = StakePool::try_from_slice(&stake_pool_info.data.borrow())?;
    let mut user_stake = UserStake::try_from_slice(&user_stake_account_info.data.borrow())?;
    
    // Validate stake ownership
    if user_stake.owner != *user_info.key {
        return Err(StakingError::Unauthorized.into());
    }
    
    // Verify stake is for this pool
    if user_stake.pool != *stake_pool_info.key {
        return Err(StakingError::InvalidStakePool.into());
    }
    
    // Validate token accounts
    if stake_pool.pool_token_account != *pool_token_account_info.key {
        return Err(StakingError::InvalidTokenAccount.into());
    }
    
    if stake_pool.pool_reward_account != *pool_reward_account_info.key {
        return Err(StakingError::InvalidTokenAccount.into());
    }
    
    // Determine amount to unstake (0 = all)
    let unstake_amount = if amount == 0 { user_stake.stake_amount } else { amount };
    
    // Validate unstake amount
    if unstake_amount > user_stake.stake_amount {
        return Err(StakingError::InsufficientStake.into());
    }
    
    // Calculate current time
    let current_time = Clock::get()?.unix_timestamp as u64;
    
    // Calculate rewards
    let rewards = calculate_rewards(&user_stake, &stake_pool, current_time);
    
    // Check if early withdrawal penalty applies
    let mut penalty_amount = 0;
    if current_time < user_stake.unlock_timestamp {
        penalty_amount = unstake_amount
            .checked_mul(stake_pool.early_withdrawal_penalty as u64)
            .ok_or(StakingError::NumericalOverflow)?
            .checked_div(10000)
            .ok_or(StakingError::NumericalOverflow)?;
    }
    
    // Transfer principal minus penalty
    let transfer_amount = unstake_amount.checked_sub(penalty_amount).ok_or(StakingError::NumericalOverflow)?;
    
    invoke(
        &spl_token::instruction::transfer(
            token_program_info.key,
            pool_token_account_info.key,
            user_token_account_info.key,
            &stake_pool.authority,
            &[],
            transfer_amount,
        )?,
        &[
            pool_token_account_info.clone(),
            user_token_account_info.clone(),
            token_program_info.clone(),
            // Note: This would require a PDA sign in real implementation
        ],
    )?;
    
    // Transfer rewards if available
    if rewards > 0 && stake_pool.reward_funds_available >= rewards {
        invoke(
            &spl_token::instruction::transfer(
                token_program_info.key,
                pool_reward_account_info.key,
                user_reward_account_info.key,
                &stake_pool.authority,
                &[],
                rewards,
            )?,
            &[
                pool_reward_account_info.clone(),
                user_reward_account_info.clone(),
                token_program_info.clone(),
                // Note: This would require a PDA sign in real implementation
            ],
        )?;
        
        // Update stake pool rewards
        stake_pool.reward_funds_available = stake_pool.reward_funds_available.checked_sub(rewards).ok_or(StakingError::NumericalOverflow)?;
        stake_pool.total_rewards_distributed = stake_pool.total_rewards_distributed.checked_add(rewards).ok_or(StakingError::NumericalOverflow)?;
    }
    
    // Update user stake
    if unstake_amount == user_stake.stake_amount {
        // Complete unstake - close account
        // In reality, we would transfer lamports back to the user
        // We'll just update for now
        user_stake.stake_amount = 0;
    } else {
        // Partial unstake
        user_stake.stake_amount = user_stake.stake_amount.checked_sub(unstake_amount).ok_or(StakingError::NumericalOverflow)?;
    }
    
    user_stake.rewards_claimed = user_stake.rewards_claimed.checked_add(rewards).ok_or(StakingError::NumericalOverflow)?;
    user_stake.last_claim_timestamp = current_time;
    
    // Update stake pool
    stake_pool.total_staked = stake_pool.total_staked.checked_sub(unstake_amount).ok_or(StakingError::NumericalOverflow)?;
    stake_pool.last_updated_timestamp = current_time;
    
    // Save updated data
    user_stake.serialize(&mut *user_stake_account_info.data.borrow_mut())?;
    stake_pool.serialize(&mut *stake_pool_info.data.borrow_mut())?;
    
    Ok(())
}

/// Process ClaimRewards instruction
fn process_claim_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let user_info = next_account_info(account_info_iter)?;
    let stake_pool_info = next_account_info(account_info_iter)?;
    let user_stake_account_info = next_account_info(account_info_iter)?;
    let pool_reward_account_info = next_account_info(account_info_iter)?;
    let user_reward_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    
    // Check the user is a signer
    if !user_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate stake pool account
    assert_owned_by(stake_pool_info, program_id)?;
    
    // Validate user stake account
    assert_owned_by(user_stake_account_info, program_id)?;
    
    // Deserialize the stake pool and user stake
    let mut stake_pool = StakePool::try_from_slice(&stake_pool_info.data.borrow())?;
    let mut user_stake = UserStake::try_from_slice(&user_stake_account_info.data.borrow())?;
    
    // Validate stake ownership
    if user_stake.owner != *user_info.key {
        return Err(StakingError::Unauthorized.into());
    }
    
    // Verify stake is for this pool
    if user_stake.pool != *stake_pool_info.key {
        return Err(StakingError::InvalidStakePool.into());
    }
    
    // Validate token accounts
    if stake_pool.pool_reward_account != *pool_reward_account_info.key {
        return Err(StakingError::InvalidTokenAccount.into());
    }
    
    // Calculate current time
    let current_time = Clock::get()?.unix_timestamp as u64;
    
    // Calculate rewards
    let rewards = calculate_rewards(&user_stake, &stake_pool, current_time);
    
    // Verify rewards are available
    if rewards == 0 {
        return Err(StakingError::InsufficientFunds.into());
    }
    
    if stake_pool.reward_funds_available < rewards {
        return Err(StakingError::InsufficientFunds.into());
    }
    
    // Transfer rewards
    invoke(
        &spl_token::instruction::transfer(
            token_program_info.key,
            pool_reward_account_info.key,
            user_reward_account_info.key,
            &stake_pool.authority,
            &[],
            rewards,
        )?,
        &[
            pool_reward_account_info.clone(),
            user_reward_account_info.clone(),
            token_program_info.clone(),
            // Note: This would require a PDA sign in real implementation
        ],
    )?;
    
    // Update stake pool rewards
    stake_pool.reward_funds_available = stake_pool.reward_funds_available.checked_sub(rewards).ok_or(StakingError::NumericalOverflow)?;
    stake_pool.total_rewards_distributed = stake_pool.total_rewards_distributed.checked_add(rewards).ok_or(StakingError::NumericalOverflow)?;
    
    // Update user stake
    user_stake.rewards_claimed = user_stake.rewards_claimed.checked_add(rewards).ok_or(StakingError::NumericalOverflow)?;
    user_stake.last_claim_timestamp = current_time;
    
    // Save updated data
    user_stake.serialize(&mut *user_stake_account_info.data.borrow_mut())?;
    stake_pool.serialize(&mut *stake_pool_info.data.borrow_mut())?;
    
    Ok(())
}

/// Process UpdatePool instruction
fn process_update_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    reward_rate: u64,
    min_stake_duration: u64,
    early_withdrawal_penalty: u16,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let authority_info = next_account_info(account_info_iter)?;
    let stake_pool_info = next_account_info(account_info_iter)?;
    
    // Check the authority is a signer
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate stake pool account
    assert_owned_by(stake_pool_info, program_id)?;
    
    // Deserialize the stake pool
    let mut stake_pool = StakePool::try_from_slice(&stake_pool_info.data.borrow())?;
    
    // Verify authority
    if stake_pool.authority != *authority_info.key {
        return Err(StakingError::Unauthorized.into());
    }
    
    // Validate reward rate
    if reward_rate == 0 {
        return Err(StakingError::InvalidRewardRate.into());
    }
    
    // Validate min stake duration
    if min_stake_duration == 0 {
        return Err(StakingError::InvalidStakeDuration.into());
    }
    
    // Validate early withdrawal penalty (max 100%)
    if early_withdrawal_penalty > 10000 {
        return Err(StakingError::InvalidRewardRate.into());
    }
    
    // Update pool parameters
    stake_pool.reward_rate = reward_rate;
    stake_pool.min_stake_duration = min_stake_duration;
    stake_pool.early_withdrawal_penalty = early_withdrawal_penalty;
    stake_pool.last_updated_timestamp = Clock::get()?.unix_timestamp as u64;
    
    // Save updated data
    stake_pool.serialize(&mut *stake_pool_info.data.borrow_mut())?;
    
    Ok(())
}

/// Process FundRewards instruction
fn process_fund_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let funder_info = next_account_info(account_info_iter)?;
    let stake_pool_info = next_account_info(account_info_iter)?;
    let funder_token_account_info = next_account_info(account_info_iter)?;
    let pool_reward_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    
    // Check the funder is a signer
    if !funder_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate stake pool account
    assert_owned_by(stake_pool_info, program_id)?;
    
    // Deserialize the stake pool
    let mut stake_pool = StakePool::try_from_slice(&stake_pool_info.data.borrow())?;
    
    // Validate token account
    if stake_pool.pool_reward_account != *pool_reward_account_info.key {
        return Err(StakingError::InvalidTokenAccount.into());
    }
    
    // Validate amount
    if amount == 0 {
        return Err(StakingError::InsufficientFunds.into());
    }
    
    // Transfer tokens from funder to pool reward account
    invoke(
        &spl_token::instruction::transfer(
            token_program_info.key,
            funder_token_account_info.key,
            pool_reward_account_info.key,
            funder_info.key,
            &[],
            amount,
        )?,
        &[
            funder_token_account_info.clone(),
            pool_reward_account_info.clone(),
            funder_info.clone(),
            token_program_info.clone(),
        ],
    )?;
    
    // Update pool reward funds
    stake_pool.reward_funds_available = stake_pool.reward_funds_available.checked_add(amount).ok_or(StakingError::NumericalOverflow)?;
    
    // Save updated data
    stake_pool.serialize(&mut *stake_pool_info.data.borrow_mut())?;
    
    Ok(())
}

/// Calculate rewards for a user stake
fn calculate_rewards(
    user_stake: &UserStake,
    stake_pool: &StakePool,
    current_time: u64,
) -> u64 {
    // Calculate time difference since last claim, capped to avoid overflows
    let time_since_last_claim = current_time.saturating_sub(user_stake.last_claim_timestamp);
    
    // Formula: rewards = stake_amount * reward_rate * time_since_last_claim / (10000 * seconds_in_day)
    // This assumes reward_rate is in basis points per day
    let seconds_in_day = 86400;
    
    let rewards = user_stake
        .stake_amount
        .checked_mul(stake_pool.reward_rate)
        .and_then(|result| result.checked_mul(time_since_last_claim))
        .and_then(|result| result.checked_div(10000))
        .and_then(|result| result.checked_div(seconds_in_day))
        .unwrap_or(0);
    
    rewards
}
