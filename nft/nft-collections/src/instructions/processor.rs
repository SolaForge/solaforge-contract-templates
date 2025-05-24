//! Program instruction processor

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    errors::TemplateError,
    instructions::TemplateInstruction,
    state::TemplateAccount,
};

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = TemplateInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        TemplateInstruction::Initialize { param1, param2 } => {
            msg!("Instruction: Initialize");
            process_initialize(program_id, accounts, param1, param2)
        }
        TemplateInstruction::Operation1 { amount } => {
            msg!("Instruction: Operation1");
            process_operation1(program_id, accounts, amount)
        }
        TemplateInstruction::Operation2 => {
            msg!("Instruction: Operation2");
            process_operation2(program_id, accounts)
        }
    }
}

/// Processes an Initialize instruction
fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    param1: u64,
    param2: u8,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let authority_info = next_account_info(account_info_iter)?;
    let account_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    
    // Check the authority is a signer
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Check account ownership
    if account_info.owner != program_id {
        return Err(TemplateError::InvalidAuthority.into());
    }
    
    // Check for rent exemption
    let rent = &Rent::from_account_info(rent_info)?;
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        return Err(TemplateError::NotRentExempt.into());
    }
    
    // Initialize account data
    let template_account = TemplateAccount {
        authority: *authority_info.key,
        value1: param1,
        value2: param2,
        is_initialized: true,
    };
    
    // Save account data
    template_account.serialize(&mut *account_info.data.borrow_mut())?;
    
    Ok(())
}

/// Processes an Operation1 instruction
fn process_operation1(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let authority_info = next_account_info(account_info_iter)?;
    let account_info = next_account_info(account_info_iter)?;
    
    // Check the authority is a signer
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Check account ownership
    if account_info.owner != program_id {
        return Err(TemplateError::InvalidAuthority.into());
    }
    
    // Deserialize account data
    let mut template_account = TemplateAccount::try_from_slice(&account_info.data.borrow())?;
    
    // Verify authority
    if template_account.authority != *authority_info.key {
        return Err(TemplateError::InvalidAuthority.into());
    }
    
    // Update account data
    template_account.value1 = template_account.value1.checked_add(amount)
        .ok_or(TemplateError::MathOverflow)?;
    
    // Save updated account data
    template_account.serialize(&mut *account_info.data.borrow_mut())?;
    
    Ok(())
}

/// Processes an Operation2 instruction
fn process_operation2(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let authority_info = next_account_info(account_info_iter)?;
    let account_info = next_account_info(account_info_iter)?;
    
    // Check the authority is a signer
    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Check account ownership
    if account_info.owner != program_id {
        return Err(TemplateError::InvalidAuthority.into());
    }
    
    // Deserialize account data
    let mut template_account = TemplateAccount::try_from_slice(&account_info.data.borrow())?;
    
    // Verify authority
    if template_account.authority != *authority_info.key {
        return Err(TemplateError::InvalidAuthority.into());
    }
    
    // Update account data
    template_account.value2 = template_account.value2.checked_add(1)
        .ok_or(TemplateError::MathOverflow)?;
    
    // Save updated account data
    template_account.serialize(&mut *account_info.data.borrow_mut())?;
    
    Ok(())
}
