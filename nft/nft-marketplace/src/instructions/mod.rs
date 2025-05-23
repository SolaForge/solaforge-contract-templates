//! Instruction types

pub mod processor;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

/// Instructions supported by the NFT Marketplace program
#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq)]
pub enum MarketplaceInstruction {
    /// Initialize the marketplace with fees and treasury
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The authority account creating this marketplace
    /// 1. `[writable]` The marketplace account to initialize
    /// 2. `[]` The treasury account to receive fees
    /// 3. `[]` The system program
    /// 4. `[]` The rent sysvar
    ///
    InitializeMarketplace {
        /// Fee basis points (e.g., 250 = 2.5%)
        fee_basis_points: u16,
    },

    /// List an NFT for sale
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The seller's account
    /// 1. `[writable]` The listing account to be created
    /// 2. `[]` The NFT mint account
    /// 3. `[writable]` The seller's NFT token account
    /// 4. `[]` The marketplace account
    /// 5. `[]` Token program
    /// 6. `[]` The system program
    /// 7. `[]` The rent sysvar
    ///
    ListNFT {
        /// Price in lamports
        price: u64,
    },

    /// Buy a listed NFT
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The buyer's account
    /// 1. `[writable]` The listing account
    /// 2. `[]` The NFT mint account
    /// 3. `[writable]` The seller's NFT token account
    /// 4. `[writable]` The buyer's NFT token account
    /// 5. `[writable]` The seller's wallet account (to receive funds)
    /// 6. `[writable]` The marketplace account
    /// 7. `[writable]` The treasury account (to receive fees)
    /// 8. `[]` Token program
    /// 9. `[]` The system program
    ///
    BuyNFT,

    /// Cancel a listing
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The seller's account
    /// 1. `[writable]` The listing account to cancel
    /// 2. `[]` The NFT mint account
    /// 3. `[writable]` The seller's NFT token account
    /// 4. `[]` The marketplace account
    /// 5. `[]` Token program
    ///
    CancelListing,

    /// Update marketplace fees
    ///
    /// Accounts expected:
    /// 0. `[writable, signer]` The marketplace authority account
    /// 1. `[writable]` The marketplace account
    ///
    UpdateMarketplaceFees {
        /// New fee basis points
        fee_basis_points: u16,
    },
}

/// Creates an instruction to initialize a marketplace
pub fn initialize_marketplace(
    program_id: &Pubkey,
    authority: &Pubkey,
    marketplace_account: &Pubkey,
    treasury_account: &Pubkey,
    fee_basis_points: u16,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*marketplace_account, false),
        AccountMeta::new_readonly(*treasury_account, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    let data = MarketplaceInstruction::InitializeMarketplace { fee_basis_points };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an instruction to list an NFT
pub fn list_nft(
    program_id: &Pubkey,
    seller: &Pubkey,
    listing_account: &Pubkey,
    nft_mint: &Pubkey,
    seller_token_account: &Pubkey,
    marketplace_account: &Pubkey,
    price: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*seller, true),
        AccountMeta::new(*listing_account, false),
        AccountMeta::new_readonly(*nft_mint, false),
        AccountMeta::new(*seller_token_account, false),
        AccountMeta::new_readonly(*marketplace_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    let data = MarketplaceInstruction::ListNFT { price };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an instruction to buy an NFT
pub fn buy_nft(
    program_id: &Pubkey,
    buyer: &Pubkey,
    listing_account: &Pubkey,
    nft_mint: &Pubkey,
    seller_token_account: &Pubkey,
    buyer_token_account: &Pubkey,
    seller_wallet: &Pubkey,
    marketplace_account: &Pubkey,
    treasury_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*buyer, true),
        AccountMeta::new(*listing_account, false),
        AccountMeta::new_readonly(*nft_mint, false),
        AccountMeta::new(*seller_token_account, false),
        AccountMeta::new(*buyer_token_account, false),
        AccountMeta::new(*seller_wallet, false),
        AccountMeta::new(*marketplace_account, false),
        AccountMeta::new(*treasury_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    let data = MarketplaceInstruction::BuyNFT;

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an instruction to cancel a listing
pub fn cancel_listing(
    program_id: &Pubkey,
    seller: &Pubkey,
    listing_account: &Pubkey,
    nft_mint: &Pubkey,
    seller_token_account: &Pubkey,
    marketplace_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*seller, true),
        AccountMeta::new(*listing_account, false),
        AccountMeta::new_readonly(*nft_mint, false),
        AccountMeta::new(*seller_token_account, false),
        AccountMeta::new_readonly(*marketplace_account, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let data = MarketplaceInstruction::CancelListing;

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}

/// Creates an instruction to update marketplace fees
pub fn update_marketplace_fees(
    program_id: &Pubkey,
    authority: &Pubkey,
    marketplace_account: &Pubkey,
    fee_basis_points: u16,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*authority, true),
        AccountMeta::new(*marketplace_account, false),
    ];

    let data = MarketplaceInstruction::UpdateMarketplaceFees { fee_basis_points };

    Instruction {
        program_id: *program_id,
        accounts,
        data: borsh::to_vec(&data).unwrap(),
    }
}
