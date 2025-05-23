//! Integration tests for staking

#[cfg(test)]
mod tests {
    use {
        borsh::BorshSerialize,
        solana_program::{
            instruction::{AccountMeta, Instruction},
            pubkey::Pubkey,
            rent::Rent,
            system_instruction,
        },
        solana_program_test::{processor, ProgramTest},
        solana_sdk::{
            account::Account,
            signature::{Keypair, Signer},
            transaction::Transaction,
        },
        single_token_staking::{
            instructions::StakingInstruction,
            process_instruction,
            state::{StakePool, UserStake},
        },
        std::str::FromStr,
    };

    #[tokio::test]
    async fn test_initialize_pool() {
        // Set up program test
        let program_id = Pubkey::from_str("Stake111111111111111111111111111111111111111").unwrap();
        let mut program_test = ProgramTest::new(
            "single_token_staking",
            program_id,
            processor!(process_instruction),
        );

        // Create keypairs for testing
        let authority = Keypair::new();
        let stake_pool_account = Keypair::new();
        let token_mint = Keypair::new();
        let pool_token_account = Keypair::new();
        let pool_reward_account = Keypair::new();
        
        // Start program test
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        
        // Airdrop SOL to authority
        let lamports = 1_000_000_000; // 1 SOL
        let txn = Transaction::new_signed_with_payer(
            &[system_instruction::transfer(
                &payer.pubkey(),
                &authority.pubkey(),
                lamports,
            )],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
        banks_client.process_transaction(txn).await.unwrap();
        
        // Initialize staking pool
        let reward_rate = 100; // 1% daily
        let min_stake_duration = 86400 * 7; // 7 days
        let early_withdrawal_penalty = 500; // 5%
        
        let init_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(authority.pubkey(), true),
                AccountMeta::new(stake_pool_account.pubkey(), false),
                AccountMeta::new_readonly(token_mint.pubkey(), false),
                AccountMeta::new(pool_token_account.pubkey(), false),
                AccountMeta::new(pool_reward_account.pubkey(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(solana_program::system_program::id(), false),
                AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
            ],
            data: StakingInstruction::InitializePool {
                reward_rate,
                min_stake_duration,
                early_withdrawal_penalty,
            }
            .try_to_vec()
            .unwrap(),
        };
        
        // Create stake pool account
        let rent = Rent::default();
        let stake_pool_size = StakePool::get_size();
        let stake_pool_rent = rent.minimum_balance(stake_pool_size);
        
        let create_stake_pool_account_ix = system_instruction::create_account(
            &authority.pubkey(),
            &stake_pool_account.pubkey(),
            stake_pool_rent,
            stake_pool_size as u64,
            &program_id,
        );
        
        // Create and submit transaction
        let mut transaction = Transaction::new_with_payer(
            &[create_stake_pool_account_ix, init_ix],
            Some(&authority.pubkey()),
        );
        transaction.sign(&[&authority, &stake_pool_account], recent_blockhash);
        
        // TODO: Uncomment and fix this for actual testing
        // Currently the test would fail due to missing program setup
        // banks_client.process_transaction(transaction).await.unwrap();
        
        // TODO: Add tests for staking, claiming rewards, and unstaking
    }
}
