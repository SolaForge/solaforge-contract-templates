//! State objects for NFT marketplace

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Status of an NFT listing
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum ListingStatus {
    /// Listing is active
    Active,
    /// Listing has been sold
    Sold,
    /// Listing has been canceled
    Canceled,
}

/// NFT Listing data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct NFTListing {
    /// Owner/Seller of the NFT
    pub seller: Pubkey,
    /// The NFT mint
    pub nft_mint: Pubkey,
    /// The seller's token account
    pub seller_token_account: Pubkey,
    /// Price in lamports
    pub price: u64,
    /// Status of the listing
    pub status: ListingStatus,
}

impl NFTListing {
    /// Get the size of NFTListing struct
    pub fn get_size() -> usize {
        // Pubkey (32 bytes) * 3 + price (8 bytes) + status (1 byte) + some padding
        32 * 3 + 8 + 1 + 8
    }
}

/// Marketplace data
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Marketplace {
    /// Authority that can update the marketplace
    pub authority: Pubkey,
    /// Treasury account to receive fees
    pub treasury: Pubkey,
    /// Fee in basis points (e.g., 250 = 2.5%)
    pub fee_basis_points: u16,
    /// Total volume transacted
    pub total_volume: u64,
    /// Total number of listings (including sold and canceled)
    pub total_listings: u64,
    /// Number of active listings
    pub active_listings: u64,
}

impl Marketplace {
    /// Get the size of Marketplace struct
    pub fn get_size() -> usize {
        // Pubkey (32 bytes) * 2 + fee_basis_points (2 bytes) + total_volume (8 bytes) +
        // total_listings (8 bytes) + active_listings (8 bytes) + some padding
        32 * 2 + 2 + 8 + 8 + 8 + 8
    }
}
