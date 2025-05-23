//! Integration tests for basic SPL token

#[cfg(test)]
mod tests {
    use {
        borsh::BorshSerialize,
        solana_program::{
            instruction::{AccountMeta, Instruction},
            program_pack::Pack,
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
        spl_token_basic::{
            instruction::TokenInstruction,
            processor::process_instruction,
            state::{Mint, TokenAccount},
        },
        std::str::FromStr,
    };

    #[tokio::test]
    async fn test_token_initialize_mint() {
        // Set up program test
        let program_id = Pubkey::from_str("TokenProg1111111111111111111111111111111111").unwrap();
        let mut program_test = ProgramTest::new(
            "spl_token_basic",
            program_id,
            processor!(process_instruction),
        );

        // Add accounts
        let mint_keypair = Keypair::new();
        let mint_authority = Keypair::new();
        
        // Start program test
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        
        // Create mint account
        let rent = Rent::default();
        let mint_account_rent = rent.minimum_balance(Mint::default().try_to_vec().unwrap().len());
        
        let create_mint_account_ix = system_instruction::create_account(
            &payer.pubkey(),
            &mint_keypair.pubkey(),
            mint_account_rent,
            Mint::default().try_to_vec().unwrap().len() as u64,
            &program_id,
        );
        
        // Initialize mint
        let init_mint_ix = Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(mint_keypair.pubkey(), false),
                AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(mint_authority.pubkey(), true),
            ],
            data: TokenInstruction::InitializeMint {
                name: "Test Token".to_string(),
                symbol: "TEST".to_string(),
                decimals: 9,
            }
            .try_to_vec()
            .unwrap(),
        };
        
        // Create and submit transaction
        let mut transaction = Transaction::new_with_payer(
            &[create_mint_account_ix, init_mint_ix],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &mint_keypair, &mint_authority], recent_blockhash);
        
        // Process transaction
        banks_client.process_transaction(transaction).await.unwrap();
        
        // Verify mint account
        let mint_account = banks_client.get_account(mint_keypair.pubkey()).await.unwrap().unwrap();
        assert_eq!(mint_account.owner, program_id);
        
        // TODO: Add more tests for token account initialization, minting and transfers
    }
}
