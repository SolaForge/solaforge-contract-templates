//! Program instruction processor

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
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
    errors::MarketplaceError,
    instructions::MarketplaceInstruction,
    state::{ListingStatus, Marketplace, NFTListing},
    utils::assert_owned_by,
};

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MarketplaceInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        MarketplaceInstruction::InitializeMarketplace { fee_basis_points } => {
            msg!("Instruction: Initialize Marketplace");
            process_initialize_marketplace(program_id, accounts, fee_basis_points)
        }
        MarketplaceInstruction::ListNFT { price } => {
            msg!("Instruction: List NFT");
            process_list_nft(program_id, accounts, price)
        }
        MarketplaceInstruction::BuyNFT => {
            msg!("Instruction: Buy NFT");
            process_buy_nft(program_id, accounts)
        }
        MarketplaceInstruction::CancelListing => {
            msg!("Instruction: Cancel Listing");
            process_cancel_listing(program_id, accounts)
        }
        MarketplaceInstruction::UpdateMarketplaceFees { fee_basis_points } => {
            msg!("Instruction: Update Marketplace Fees");
            process_update_marketplace_fees(program_id, accounts, fee_basis_points)
        }
    }
}

/// Processes a InitializeMarketplace instruction
fn process_initialize_marketplace(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fee_basis_points: u16,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let authority_info = next_account_info(account_info_iter)?;
    let marketplace_account_info = next_account_info(account_info_iter)?;
    let treasury_account_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    
    // Check the authority is a signer
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate fee basis points (max 10%)
    if fee_basis_points > 1000 {
        return Err(MarketplaceError::InvalidListingPrice.into());
    }
    
    // Create marketplace account
    let rent = &Rent::from_account_info(rent_info)?;
    let marketplace_size = Marketplace::get_size();
    let marketplace_lamports = rent.minimum_balance(marketplace_size);
    
    invoke(
        &system_instruction::create_account(
            authority_info.key,
            marketplace_account_info.key,
            marketplace_lamports,
            marketplace_size as u64,
            program_id,
        ),
        &[
            authority_info.clone(),
            marketplace_account_info.clone(),
            system_program_info.clone(),
        ],
    )?;
    
    // Initialize marketplace data
    let marketplace = Marketplace {
        authority: *authority_info.key,
        treasury: *treasury_account_info.key,
        fee_basis_points,
        total_volume: 0,
        total_listings: 0,
        active_listings: 0,
    };
    
    marketplace.serialize(&mut *marketplace_account_info.data.borrow_mut())?;
    
    Ok(())
}

/// Processes a ListNFT instruction
fn process_list_nft(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    price: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let seller_info = next_account_info(account_info_iter)?;
    let listing_account_info = next_account_info(account_info_iter)?;
    let nft_mint_info = next_account_info(account_info_iter)?;
    let seller_token_account_info = next_account_info(account_info_iter)?;
    let marketplace_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    
    // Check the seller is a signer
    if !seller_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify price is valid
    if price == 0 {
        return Err(MarketplaceError::InvalidListingPrice.into());
    }
    
    // Verify token account ownership
    let token_account = spl_token::state::Account::unpack(&seller_token_account_info.data.borrow())?;
    if token_account.owner != *seller_info.key {
        return Err(MarketplaceError::NotNFTOwner.into());
    }
    
    // Verify token account is for the right mint
    if token_account.mint != *nft_mint_info.key {
        return Err(MarketplaceError::NFTAccountMismatch.into());
    }
    
    // Verify token account has exactly 1 token (it's an NFT)
    if token_account.amount != 1 {
        return Err(MarketplaceError::NFTAccountMismatch.into());
    }
    
    // Create listing account
    let rent = &Rent::from_account_info(rent_info)?;
    let listing_size = NFTListing::get_size();
    let listing_lamports = rent.minimum_balance(listing_size);
    
    invoke(
        &system_instruction::create_account(
            seller_info.key,
            listing_account_info.key,
            listing_lamports,
            listing_size as u64,
            program_id,
        ),
        &[
            seller_info.clone(),
            listing_account_info.clone(),
            system_program_info.clone(),
        ],
    )?;
    
    // Initialize listing data
    let listing = NFTListing {
        seller: *seller_info.key,
        nft_mint: *nft_mint_info.key,
        seller_token_account: *seller_token_account_info.key,
        price,
        status: ListingStatus::Active,
    };
    
    listing.serialize(&mut *listing_account_info.data.borrow_mut())?;
    
    // Update marketplace data
    let mut marketplace = Marketplace::try_from_slice(&marketplace_account_info.data.borrow())?;
    marketplace.total_listings = marketplace.total_listings.checked_add(1).ok_or(MarketplaceError::NumericalOverflow)?;
    marketplace.active_listings = marketplace.active_listings.checked_add(1).ok_or(MarketplaceError::NumericalOverflow)?;
    marketplace.serialize(&mut *marketplace_account_info.data.borrow_mut())?;
    
    Ok(())
}

/// Processes a BuyNFT instruction
fn process_buy_nft(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let buyer_info = next_account_info(account_info_iter)?;
    let listing_account_info = next_account_info(account_info_iter)?;
    let nft_mint_info = next_account_info(account_info_iter)?;
    let seller_token_account_info = next_account_info(account_info_iter)?;
    let buyer_token_account_info = next_account_info(account_info_iter)?;
    let seller_wallet_info = next_account_info(account_info_iter)?;
    let marketplace_account_info = next_account_info(account_info_iter)?;
    let treasury_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    
    // Check the buyer is a signer
    if !buyer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify listing account is owned by program
    assert_owned_by(listing_account_info, program_id)?;
    
    // Verify marketplace account is owned by program
    assert_owned_by(marketplace_account_info, program_id)?;
    
    // Get listing data
    let mut listing = NFTListing::try_from_slice(&listing_account_info.data.borrow())?;
    
    // Verify listing is active
    if listing.status != ListingStatus::Active {
        return Err(MarketplaceError::ListingNotActive.into());
    }
    
    // Verify token accounts match the listing
    if listing.nft_mint != *nft_mint_info.key {
        return Err(MarketplaceError::NFTAccountMismatch.into());
    }
    
    if listing.seller_token_account != *seller_token_account_info.key {
        return Err(MarketplaceError::NFTAccountMismatch.into());
    }
    
    // Verify token accounts
    let seller_token = spl_token::state::Account::unpack(&seller_token_account_info.data.borrow())?;
    if seller_token.mint != *nft_mint_info.key || seller_token.amount != 1 {
        return Err(MarketplaceError::NFTAccountMismatch.into());
    }
    
    // Get marketplace data
    let mut marketplace = Marketplace::try_from_slice(&marketplace_account_info.data.borrow())?;
    
    // Verify treasury account
    if marketplace.treasury != *treasury_account_info.key {
        return Err(MarketplaceError::InvalidTreasuryAccount.into());
    }
    
    // Calculate fees
    let fee_amount = listing.price
        .checked_mul(marketplace.fee_basis_points as u64)
        .ok_or(MarketplaceError::NumericalOverflow)?
        .checked_div(10000)
        .ok_or(MarketplaceError::NumericalOverflow)?;
    
    let seller_amount = listing.price.checked_sub(fee_amount).ok_or(MarketplaceError::NumericalOverflow)?;
    
    // Transfer SOL to seller
    invoke(
        &system_instruction::transfer(buyer_info.key, seller_wallet_info.key, seller_amount),
        &[
            buyer_info.clone(),
            seller_wallet_info.clone(),
            system_program_info.clone(),
        ],
    )?;
    
    // Transfer fees to treasury
    invoke(
        &system_instruction::transfer(buyer_info.key, treasury_account_info.key, fee_amount),
        &[
            buyer_info.clone(),
            treasury_account_info.clone(),
            system_program_info.clone(),
        ],
    )?;
    
    // Transfer NFT to buyer
    invoke(
        &spl_token::instruction::transfer(
            token_program_info.key,
            seller_token_account_info.key,
            buyer_token_account_info.key,
            &listing.seller,
            &[],
            1,
        )?,
        &[
            seller_token_account_info.clone(),
            buyer_token_account_info.clone(),
            token_program_info.clone(),
            // Note: We need authority but the program will check the seller account 
            // which we don't have a signature for
            // This would need a different flow in practice
        ],
    )?;
    
    // Update listing status
    listing.status = ListingStatus::Sold;
    listing.serialize(&mut *listing_account_info.data.borrow_mut())?;
    
    // Update marketplace stats
    marketplace.active_listings = marketplace.active_listings.checked_sub(1).ok_or(MarketplaceError::NumericalOverflow)?;
    marketplace.total_volume = marketplace.total_volume.checked_add(listing.price).ok_or(MarketplaceError::NumericalOverflow)?;
    marketplace.serialize(&mut *marketplace_account_info.data.borrow_mut())?;
    
    Ok(())
}

/// Processes a CancelListing instruction
fn process_cancel_listing(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let seller_info = next_account_info(account_info_iter)?;
    let listing_account_info = next_account_info(account_info_iter)?;
    let nft_mint_info = next_account_info(account_info_iter)?;
    let seller_token_account_info = next_account_info(account_info_iter)?;
    let marketplace_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    
    // Check the seller is a signer
    if !seller_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify listing account is owned by program
    assert_owned_by(listing_account_info, program_id)?;
    
    // Verify marketplace account is owned by program
    assert_owned_by(marketplace_account_info, program_id)?;
    
    // Get listing data
    let mut listing = NFTListing::try_from_slice(&listing_account_info.data.borrow())?;
    
    // Verify listing belongs to seller
    if listing.seller != *seller_info.key {
        return Err(MarketplaceError::AuthorityMismatch.into());
    }
    
    // Verify listing is active
    if listing.status != ListingStatus::Active {
        return Err(MarketplaceError::ListingNotActive.into());
    }
    
    // Verify token accounts match the listing
    if listing.nft_mint != *nft_mint_info.key {
        return Err(MarketplaceError::NFTAccountMismatch.into());
    }
    
    if listing.seller_token_account != *seller_token_account_info.key {
        return Err(MarketplaceError::NFTAccountMismatch.into());
    }
    
    // Update listing status
    listing.status = ListingStatus::Canceled;
    listing.serialize(&mut *listing_account_info.data.borrow_mut())?;
    
    // Update marketplace stats
    let mut marketplace = Marketplace::try_from_slice(&marketplace_account_info.data.borrow())?;
    marketplace.active_listings = marketplace.active_listings.checked_sub(1).ok_or(MarketplaceError::NumericalOverflow)?;
    marketplace.serialize(&mut *marketplace_account_info.data.borrow_mut())?;
    
    Ok(())
}

/// Processes an UpdateMarketplaceFees instruction
fn process_update_marketplace_fees(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    fee_basis_points: u16,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let authority_info = next_account_info(account_info_iter)?;
    let marketplace_account_info = next_account_info(account_info_iter)?;
    
    // Check the authority is a signer
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify marketplace account is owned by program
    assert_owned_by(marketplace_account_info, program_id)?;
    
    // Validate fee basis points (max 10%)
    if fee_basis_points > 1000 {
        return Err(MarketplaceError::InvalidListingPrice.into());
    }
    
    // Get marketplace data
    let mut marketplace = Marketplace::try_from_slice(&marketplace_account_info.data.borrow())?;
    
    // Verify authority
    if marketplace.authority != *authority_info.key {
        return Err(MarketplaceError::AuthorityMismatch.into());
    }
    
    // Update fees
    marketplace.fee_basis_points = fee_basis_points;
    marketplace.serialize(&mut *marketplace_account_info.data.borrow_mut())?;
    
    Ok(())
}
