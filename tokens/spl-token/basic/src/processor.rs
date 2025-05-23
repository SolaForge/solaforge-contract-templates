//! Program processor

use borsh::BorshDeserialize;
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
    error::TokenError,
    instruction::TokenInstruction,
    state::{Mint, TokenAccount},
};

/// Program processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TokenInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        TokenInstruction::InitializeMint {
            name,
            symbol,
            decimals,
        } => {
            msg!("Instruction: InitializeMint");
            process_initialize_mint(program_id, accounts, name, symbol, decimals)
        }
        TokenInstruction::InitializeAccount => {
            msg!("Instruction: InitializeAccount");
            process_initialize_account(program_id, accounts)
        }
        TokenInstruction::MintTo { amount } => {
            msg!("Instruction: MintTo");
            process_mint_to(program_id, accounts, amount)
        }
        TokenInstruction::Transfer { amount } => {
            msg!("Instruction: Transfer");
            process_transfer(program_id, accounts, amount)
        }
    }
}

/// Process InitializeMint instruction
fn process_initialize_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    symbol: String,
    decimals: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let mint_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    
    // Check owner
    if mint_info.owner != program_id {
        return Err(TokenError::OwnerMismatch.into());
    }
    
    // Check for rent exemption
    let rent = &Rent::from_account_info(rent_info)?;
    if !rent.is_exempt(mint_info.lamports(), mint_info.data_len()) {
        return Err(TokenError::NotRentExempt.into());
    }
    
    // Create the mint
    let mut mint_data = Mint {
        mint_authority: Some(*mint_authority_info.key),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: None,
        name,
        symbol,
    };
    
    // Save the mint data
    mint_data.serialize(&mut *mint_info.data.borrow_mut())?;
    
    // Initialize the mint with SPL Token program
    let ix = spl_token::instruction::initialize_mint(
        token_program_info.key,
        mint_info.key,
        mint_authority_info.key,
        None,
        decimals,
    )?;
    
    invoke(
        &ix,
        &[
            mint_info.clone(),
            rent_info.clone(),
            token_program_info.clone(),
        ],
    )?;
    
    Ok(())
}

/// Process InitializeAccount instruction
fn process_initialize_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let account_info = next_account_info(account_info_iter)?;
    let mint_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    
    // Check owner
    if account_info.owner != program_id {
        return Err(TokenError::OwnerMismatch.into());
    }
    
    // Check for rent exemption
    let rent = &Rent::from_account_info(rent_info)?;
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        return Err(TokenError::NotRentExempt.into());
    }
    
    // Create token account
    let token_account = TokenAccount {
        mint: *mint_info.key,
        owner: *owner_info.key,
        amount: 0,
        is_frozen: false,
        is_initialized: true,
    };
    
    // Save token account data
    token_account.serialize(&mut *account_info.data.borrow_mut())?;
    
    // Initialize the token account with SPL Token program
    let ix = spl_token::instruction::initialize_account(
        token_program_info.key,
        account_info.key,
        mint_info.key,
        owner_info.key,
    )?;
    
    invoke(
        &ix,
        &[
            account_info.clone(),
            mint_info.clone(),
            owner_info.clone(),
            rent_info.clone(),
            token_program_info.clone(),
        ],
    )?;
    
    Ok(())
}

/// Process MintTo instruction
fn process_mint_to(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let mint_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let mint_authority_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    
    // Verify mint account
    if mint_info.owner != program_id {
        return Err(TokenError::OwnerMismatch.into());
    }
    
    // Verify destination account
    if destination_info.owner != program_id {
        return Err(TokenError::OwnerMismatch.into());
    }
    
    // Deserialize mint data
    let mut mint_data = Mint::try_from_slice(&mint_info.data.borrow())?;
    
    // Check mint authority
    if mint_data.mint_authority != Some(*mint_authority_info.key) {
        return Err(TokenError::Unauthorized.into());
    }
    
    // Deserialize destination account
    let mut dest_account = TokenAccount::try_from_slice(&destination_info.data.borrow())?;
    
    // Ensure destination is for this mint
    if dest_account.mint != *mint_info.key {
        return Err(TokenError::ExpectedMint.into());
    }
    
    // Update mint supply
    mint_data.supply = mint_data.supply.checked_add(amount).ok_or(TokenError::InsufficientFunds)?;
    
    // Update destination balance
    dest_account.amount = dest_account.amount.checked_add(amount).ok_or(TokenError::InsufficientFunds)?;
    
    // Save updated data
    mint_data.serialize(&mut *mint_info.data.borrow_mut())?;
    dest_account.serialize(&mut *destination_info.data.borrow_mut())?;
    
    // Call SPL Token program to mint tokens
    let ix = spl_token::instruction::mint_to(
        token_program_info.key,
        mint_info.key,
        destination_info.key,
        mint_authority_info.key,
        &[],
        amount,
    )?;
    
    invoke(
        &ix,
        &[
            mint_info.clone(),
            destination_info.clone(),
            mint_authority_info.clone(),
            token_program_info.clone(),
        ],
    )?;
    
    Ok(())
}

/// Process Transfer instruction
fn process_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let source_info = next_account_info(account_info_iter)?;
    let destination_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    
    // Verify accounts
    if source_info.owner != program_id {
        return Err(TokenError::OwnerMismatch.into());
    }
    
    if destination_info.owner != program_id {
        return Err(TokenError::OwnerMismatch.into());
    }
    
    // Deserialize accounts
    let mut source_account = TokenAccount::try_from_slice(&source_info.data.borrow())?;
    let mut dest_account = TokenAccount::try_from_slice(&destination_info.data.borrow())?;
    
    // Check ownership
    if source_account.owner != *owner_info.key {
        return Err(TokenError::Unauthorized.into());
    }
    
    // Ensure same mint
    if source_account.mint != dest_account.mint {
        return Err(TokenError::ExpectedMint.into());
    }
    
    // Check sufficient funds
    if source_account.amount < amount {
        return Err(TokenError::InsufficientFunds.into());
    }
    
    // Update balances
    source_account.amount = source_account.amount.checked_sub(amount).ok_or(TokenError::InsufficientFunds)?;
    dest_account.amount = dest_account.amount.checked_add(amount).ok_or(TokenError::InsufficientFunds)?;
    
    // Save updated data
    source_account.serialize(&mut *source_info.data.borrow_mut())?;
    dest_account.serialize(&mut *destination_info.data.borrow_mut())?;
    
    // Call SPL Token program to transfer tokens
    let ix = spl_token::instruction::transfer(
        token_program_info.key,
        source_info.key,
        destination_info.key,
        owner_info.key,
        &[],
        amount,
    )?;
    
    invoke(
        &ix,
        &[
            source_info.clone(),
            destination_info.clone(),
            owner_info.clone(),
            token_program_info.clone(),
        ],
    )?;
    
    Ok(())
}
