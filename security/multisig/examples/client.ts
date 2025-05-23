/**
 * Example client for Multisig Security operations
 */
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import * as borsh from 'borsh';

// Define the program ID (replace with your actual program ID)
const PROGRAM_ID = new PublicKey('MuLti51gkEJZAQYYcE5Gfx2qC4nC6YtQJLyLBzf5vPGW');

// Define instruction types
enum MultisigInstructionType {
  CreateMultisig = 0,
  CreateTransaction = 1,
  ApproveTransaction = 2,
  ExecuteTransaction = 3,
  RemoveTransaction = 4,
  ChangeOwners = 5,
}

// Define instruction schema
class CreateMultisigInstruction {
  threshold: number;
  owners: Uint8Array[];
  
  constructor(props: { threshold: number; owners: PublicKey[] }) {
    this.threshold = props.threshold;
    this.owners = props.owners.map(owner => owner.toBytes());
  }
  
  static schema = new Map([
    [
      CreateMultisigInstruction,
      {
        kind: 'struct',
        fields: [
          ['threshold', 'u8'],
          ['owners', ['[32]']],
        ],
      },
    ],
  ]);
}

class CreateTransactionInstruction {
  transactionData: Uint8Array;
  
  constructor(props: { transactionData: Uint8Array }) {
    this.transactionData = props.transactionData;
  }
  
  static schema = new Map([
    [
      CreateTransactionInstruction,
      {
        kind: 'struct',
        fields: [
          ['transactionData', ['u8']],
        ],
      },
    ],
  ]);
}

/**
 * Create a multisig account
 */
async function createMultisig(
  connection: Connection,
  funder: Keypair,
  multisigAccount: Keypair,
  owners: PublicKey[],
  threshold: number
) {
  console.log('Creating multisig with the following owners:');
  owners.forEach((owner, i) => {
    console.log(`Owner ${i + 1}: ${owner.toString()}`);
  });
  console.log(`Threshold: ${threshold} of ${owners.length}`);
  
  // Serialize CreateMultisig instruction
  const createMultisigInstruction = new CreateMultisigInstruction({
    threshold,
    owners,
  });
  
  const createMultisigData = Buffer.from([MultisigInstructionType.CreateMultisig]);
  const instructionData = borsh.serialize(
    CreateMultisigInstruction.schema,
    createMultisigInstruction
  );
  
  const fullData = Buffer.concat([createMultisigData, instructionData]);
  
  const transaction = new Transaction().add({
    keys: [
      { pubkey: funder.publicKey, isSigner: true, isWritable: true },
      { pubkey: multisigAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: fullData,
  });
  
  try {
    await sendAndConfirmTransaction(
      connection,
      transaction,
      [funder, multisigAccount]
    );
    console.log('Multisig created successfully!');
    console.log('Multisig address:', multisigAccount.publicKey.toString());
  } catch (error) {
    console.error('Error creating multisig:', error);
  }
}

/**
 * Create a transaction for the multisig to approve
 */
async function createTransaction(
  connection: Connection,
  owner: Keypair,
  multisigAccount: PublicKey,
  transactionAccount: Keypair,
  instruction: TransactionInstruction
) {
  // Serialize the instruction for the transaction data
  // This is a simplified version - a real implementation would
  // fully serialize the instruction including accounts
  const programIdBytes = instruction.programId.toBytes();
  const dataLengthBytes = Buffer.alloc(4);
  dataLengthBytes.writeUInt32LE(instruction.data.length, 0);
  
  const transactionData = Buffer.concat([
    Buffer.from(programIdBytes),
    dataLengthBytes,
    instruction.data,
  ]);
  
  // Serialize CreateTransaction instruction
  const createTransactionInstruction = new CreateTransactionInstruction({
    transactionData,
  });
  
  const createTransactionData = Buffer.from([MultisigInstructionType.CreateTransaction]);
  const instructionData = borsh.serialize(
    CreateTransactionInstruction.schema,
    createTransactionInstruction
  );
  
  const fullData = Buffer.concat([createTransactionData, instructionData]);
  
  const transaction = new Transaction().add({
    keys: [
      { pubkey: owner.publicKey, isSigner: true, isWritable: true },
      { pubkey: multisigAccount, isSigner: false, isWritable: true },
      { pubkey: transactionAccount.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: fullData,
  });
  
  try {
    await sendAndConfirmTransaction(
      connection,
      transaction,
      [owner, transactionAccount]
    );
    console.log('Transaction created successfully!');
    console.log('Transaction address:', transactionAccount.publicKey.toString());
  } catch (error) {
    console.error('Error creating transaction:', error);
  }
}

/**
 * Approve a transaction
 */
async function approveTransaction(
  connection: Connection,
  owner: Keypair,
  multisigAccount: PublicKey,
  transactionAccount: PublicKey
) {
  const approveData = Buffer.from([MultisigInstructionType.ApproveTransaction]);
  
  const transaction = new Transaction().add({
    keys: [
      { pubkey: owner.publicKey, isSigner: true, isWritable: true },
      { pubkey: multisigAccount, isSigner: false, isWritable: true },
      { pubkey: transactionAccount, isSigner: false, isWritable: true },
    ],
    programId: PROGRAM_ID,
    data: approveData,
  });
  
  try {
    await sendAndConfirmTransaction(
      connection,
      transaction,
      [owner]
    );
    console.log('Transaction approved successfully!');
  } catch (error) {
    console.error('Error approving transaction:', error);
  }
}

/**
 * Main example function
 */
async function main() {
  // Connect to devnet
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  
  // Generate keypairs for testing
  const funder = Keypair.generate();
  const multisigAccount = Keypair.generate();
  const owner1 = Keypair.generate();
  const owner2 = Keypair.generate();
  const owner3 = Keypair.generate();
  
  console.log('Requesting airdrop for funder...');
  const airdropSignature = await connection.requestAirdrop(funder.publicKey, 2 * 10**9);
  await connection.confirmTransaction(airdropSignature);
  
  // Create a 2-of-3 multisig
  await createMultisig(
    connection,
    funder,
    multisigAccount,
    [owner1.publicKey, owner2.publicKey, owner3.publicKey],
    2
  );
  
  // Create a transaction to be approved
  // In a real application, this would be an actual action to perform
  const sampleInstruction = new TransactionInstruction({
    keys: [],
    programId: SystemProgram.programId,
    data: Buffer.from('Sample transaction data'),
  });
  
  const transactionAccount = Keypair.generate();
  
  await createTransaction(
    connection,
    owner1,
    multisigAccount.publicKey,
    transactionAccount,
    sampleInstruction
  );
  
  // Approve the transaction
  await approveTransaction(
    connection,
    owner2,
    multisigAccount.publicKey,
    transactionAccount.publicKey
  );
  
  // In a real application, you would then execute the transaction
  // once it has enough approvals
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  }
);
