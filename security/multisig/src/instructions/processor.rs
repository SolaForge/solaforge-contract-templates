//! Program instruction processor

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use crate::{
    errors::MultisigError,
    instructions::MultisigInstruction,
    state::{MultisigAccount, Transaction, TransactionStatus},
    utils::assert_owned_by,
};

/// Processes an instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = MultisigInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        MultisigInstruction::CreateMultisig { threshold, owners } => {
            msg!("Instruction: Create Multisig");
            process_create_multisig(program_id, accounts, threshold, owners)
        }
        MultisigInstruction::CreateTransaction { transaction_data } => {
            msg!("Instruction: Create Transaction");
            process_create_transaction(program_id, accounts, transaction_data)
        }
        MultisigInstruction::ApproveTransaction => {
            msg!("Instruction: Approve Transaction");
            process_approve_transaction(program_id, accounts)
        }
        MultisigInstruction::ExecuteTransaction => {
            msg!("Instruction: Execute Transaction");
            process_execute_transaction(program_id, accounts)
        }
        MultisigInstruction::RemoveTransaction => {
            msg!("Instruction: Remove Transaction");
            process_remove_transaction(program_id, accounts)
        }
        MultisigInstruction::ChangeOwners { threshold, owners } => {
            msg!("Instruction: Change Owners");
            process_change_owners(program_id, accounts, threshold, owners)
        }
    }
}

/// Creates a new multisig account
fn process_create_multisig(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    threshold: u8,
    owners: Vec<Pubkey>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let funder_info = next_account_info(account_info_iter)?;
    let multisig_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    
    // Check the funder is a signer
    if !funder_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Validate the threshold
    if threshold == 0 || threshold > owners.len() as u8 {
        return Err(MultisigError::InvalidNumberOfSigners.into());
    }
    
    // Validate owners (check for duplicates)
    let mut unique_owners = owners.clone();
    unique_owners.sort();
    unique_owners.dedup();
    if unique_owners.len() != owners.len() {
        return Err(MultisigError::DuplicateOwner.into());
    }
    
    // Create the multisig account
    let rent = &Rent::from_account_info(rent_info)?;
    let multisig_account = MultisigAccount {
        is_initialized: true,
        threshold,
        owners: owners.clone(),
        transaction_count: 0,
    };
    
    // Calculate account size
    let account_size = multisig_account.get_packed_len();
    let lamports = rent.minimum_balance(account_size);
    
    // Create account with system program
    invoke(
        &system_instruction::create_account(
            funder_info.key,
            multisig_info.key,
            lamports,
            account_size as u64,
            program_id,
        ),
        &[
            funder_info.clone(),
            multisig_info.clone(),
            system_program_info.clone(),
        ],
    )?;
    
    // Save the multisig account
    multisig_account.serialize(&mut *multisig_info.data.borrow_mut())?;
    
    Ok(())
}

/// Creates a new transaction for the multisig to approve
fn process_create_transaction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    transaction_data: Vec<u8>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let owner_info = next_account_info(account_info_iter)?;
    let multisig_info = next_account_info(account_info_iter)?;
    let transaction_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;
    let rent_info = next_account_info(account_info_iter)?;
    
    // Check the owner is a signer
    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify multisig account
    assert_owned_by(multisig_info, program_id)?;
    let multisig_account = MultisigAccount::try_from_slice(&multisig_info.data.borrow())?;
    
    // Verify owner is part of the multisig
    if !multisig_account.owners.contains(owner_info.key) {
        return Err(MultisigError::OwnerNotFound.into());
    }
    
    // Create the transaction
    let mut approvers = vec![false; multisig_account.owners.len()];
    let owner_index = multisig_account
        .owners
        .iter()
        .position(|owner| owner == owner_info.key)
        .ok_or(MultisigError::OwnerNotFound)?;
    
    // Initial owner is automatically an approver
    approvers[owner_index] = true;
    
    let transaction = Transaction {
        multisig: *multisig_info.key,
        status: TransactionStatus::Active,
        transaction_data,
        signers: approvers,
        creator: *owner_info.key,
        executed_at: 0,
    };
    
    // Calculate account size
    let account_size = transaction.get_packed_len();
    let rent = &Rent::from_account_info(rent_info)?;
    let lamports = rent.minimum_balance(account_size);
    
    // Create transaction account
    invoke(
        &system_instruction::create_account(
            owner_info.key,
            transaction_info.key,
            lamports,
            account_size as u64,
            program_id,
        ),
        &[
            owner_info.clone(),
            transaction_info.clone(),
            system_program_info.clone(),
        ],
    )?;
    
    // Save the transaction
    transaction.serialize(&mut *transaction_info.data.borrow_mut())?;
    
    // Update multisig transaction count
    let mut multisig_account = MultisigAccount::try_from_slice(&multisig_info.data.borrow())?;
    multisig_account.transaction_count += 1;
    multisig_account.serialize(&mut *multisig_info.data.borrow_mut())?;
    
    Ok(())
}

/// Approves a transaction
fn process_approve_transaction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let owner_info = next_account_info(account_info_iter)?;
    let multisig_info = next_account_info(account_info_iter)?;
    let transaction_info = next_account_info(account_info_iter)?;
    
    // Check the owner is a signer
    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify multisig account
    assert_owned_by(multisig_info, program_id)?;
    let multisig_account = MultisigAccount::try_from_slice(&multisig_info.data.borrow())?;
    
    // Verify owner is part of the multisig
    if !multisig_account.owners.contains(owner_info.key) {
        return Err(MultisigError::OwnerNotFound.into());
    }
    
    // Verify transaction account
    assert_owned_by(transaction_info, program_id)?;
    let mut transaction = Transaction::try_from_slice(&transaction_info.data.borrow())?;
    
    // Verify transaction belongs to this multisig
    if transaction.multisig != *multisig_info.key {
        return Err(MultisigError::InvalidTransactionData.into());
    }
    
    // Verify transaction is active
    if transaction.status != TransactionStatus::Active {
        return Err(MultisigError::TransactionNotReady.into());
    }
    
    // Find owner index and mark as approved
    let owner_index = multisig_account
        .owners
        .iter()
        .position(|owner| owner == owner_info.key)
        .ok_or(MultisigError::OwnerNotFound)?;
    
    // Verify the owner hasn't already signed
    if transaction.signers[owner_index] {
        return Err(MultisigError::OwnerAlreadySigned.into());
    }
    
    // Mark as approved
    transaction.signers[owner_index] = true;
    
    // Save the updated transaction
    transaction.serialize(&mut *transaction_info.data.borrow_mut())?;
    
    Ok(())
}

/// Executes a transaction if it has enough approvals
fn process_execute_transaction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let owner_info = next_account_info(account_info_iter)?;
    let multisig_info = next_account_info(account_info_iter)?;
    let transaction_info = next_account_info(account_info_iter)?;
    
    // Check the owner is a signer
    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify multisig account
    assert_owned_by(multisig_info, program_id)?;
    let multisig_account = MultisigAccount::try_from_slice(&multisig_info.data.borrow())?;
    
    // Verify owner is part of the multisig
    if !multisig_account.owners.contains(owner_info.key) {
        return Err(MultisigError::OwnerNotFound.into());
    }
    
    // Verify transaction account
    assert_owned_by(transaction_info, program_id)?;
    let mut transaction = Transaction::try_from_slice(&transaction_info.data.borrow())?;
    
    // Verify transaction belongs to this multisig
    if transaction.multisig != *multisig_info.key {
        return Err(MultisigError::InvalidTransactionData.into());
    }
    
    // Verify transaction is active
    if transaction.status != TransactionStatus::Active {
        return Err(MultisigError::TransactionNotReady.into());
    }
    
    // Count approvals
    let approval_count = transaction.signers.iter().filter(|&approved| *approved).count();
    
    // Verify enough approvals
    if approval_count < multisig_account.threshold as usize {
        return Err(MultisigError::NotEnoughSigners.into());
    }
    
    // Parse the transaction data
    let transaction_data = &transaction.transaction_data;
    if transaction_data.len() < 32 {
        return Err(MultisigError::InvalidTransactionData.into());
    }
    
    // Get the program ID from the transaction data
    let program_id_bytes: [u8; 32] = transaction_data[0..32].try_into().unwrap();
    let target_program_id = Pubkey::new_from_array(program_id_bytes);
    
    // Get remaining accounts to pass to the target program
    let account_infos: Vec<AccountInfo> = account_info_iter.cloned().collect();
    
    // Construct the instruction from the transaction data
    // TODO: In a real implementation, deserialize the accounts and data from transaction_data
    // and build a proper instruction
    let instruction_data = &transaction_data[32..];
    
    // Execute the transaction via CPI
    invoke(
        &solana_program::instruction::Instruction {
            program_id: target_program_id,
            accounts: vec![], // In reality, this would come from transaction_data
            data: instruction_data.to_vec(),
        },
        &account_infos,
    )?;
    
    // Mark transaction as executed
    transaction.status = TransactionStatus::Executed;
    transaction.executed_at = solana_program::clock::Clock::get()?.unix_timestamp as u64;
    
    // Save the updated transaction
    transaction.serialize(&mut *transaction_info.data.borrow_mut())?;
    
    Ok(())
}

/// Removes a transaction
fn process_remove_transaction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let owner_info = next_account_info(account_info_iter)?;
    let multisig_info = next_account_info(account_info_iter)?;
    let transaction_info = next_account_info(account_info_iter)?;
    
    // Check the owner is a signer
    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    
    // Verify multisig account
    assert_owned_by(multisig_info, program_id)?;
    let multisig_account = MultisigAccount::try_from_slice(&multisig_info.data.borrow())?;
    
    // Verify owner is part of the multisig
    if !multisig_account.owners.contains(owner_info.key) {
        return Err(MultisigError::OwnerNotFound.into());
    }
    
    // Verify transaction account
    assert_owned_by(transaction_info, program_id)?;
    let transaction = Transaction::try_from_slice(&transaction_info.data.borrow())?;
    
    // Verify transaction belongs to this multisig
    if transaction.multisig != *multisig_info.key {
        return Err(MultisigError::InvalidTransactionData.into());
    }
    
    // Only allow creator or any owner to remove if not executed
    if transaction.creator != *owner_info.key && transaction.status != TransactionStatus::Executed {
        // For non-executed transactions, any owner can remove only if it's their own transaction
        return Err(MultisigError::AuthorityMismatch.into());
    }
    
    // Mark transaction as removed
    let mut transaction = Transaction::try_from_slice(&transaction_info.data.borrow())?;
    transaction.status = TransactionStatus::Removed;
    
    // Save the updated transaction
    transaction.serialize(&mut *transaction_info.data.borrow_mut())?;
    
    Ok(())
}

/// Changes the owners or threshold of a multisig
fn process_change_owners(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    threshold: u8,
    owners: Vec<Pubkey>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    
    // Get accounts
    let multisig_info = next_account_info(account_info_iter)?;
    let transaction_info = next_account_info(account_info_iter)?;
    
    // Verify multisig account
    assert_owned_by(multisig_info, program_id)?;
    
    // Verify transaction account
    assert_owned_by(transaction_info, program_id)?;
    let transaction = Transaction::try_from_slice(&transaction_info.data.borrow())?;
    
    // Verify transaction belongs to this multisig
    if transaction.multisig != *multisig_info.key {
        return Err(MultisigError::InvalidTransactionData.into());
    }
    
    // Verify transaction is executed
    if transaction.status != TransactionStatus::Executed {
        return Err(MultisigError::TransactionNotReady.into());
    }
    
    // Validate the threshold
    if threshold == 0 || threshold > owners.len() as u8 {
        return Err(MultisigError::InvalidNumberOfSigners.into());
    }
    
    // Validate owners (check for duplicates)
    let mut unique_owners = owners.clone();
    unique_owners.sort();
    unique_owners.dedup();
    if unique_owners.len() != owners.len() {
        return Err(MultisigError::DuplicateOwner.into());
    }
    
    // Update the multisig account
    let mut multisig_account = MultisigAccount::try_from_slice(&multisig_info.data.borrow())?;
    multisig_account.threshold = threshold;
    multisig_account.owners = owners;
    
    // Save the updated multisig account
    multisig_account.serialize(&mut *multisig_info.data.borrow_mut())?;
    
    Ok(())
}
