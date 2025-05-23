/**
 * Example client for dao-proposal operations
 */
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import * as borsh from 'borsh';
import BN from 'bn.js';

// Define the program ID (replace with your actual program ID)
const PROGRAM_ID = new PublicKey('dao-proposal111111111111111111111111111111');

// Define instruction types
enum TemplateInstructionType {
  Initialize = 0,
  Operation1 = 1,
  Operation2 = 2,
}

// Define instruction schema
class InitializeInstruction {
  param1: BN;
  param2: number;
  
  constructor(props: { param1: BN; param2: number }) {
    this.param1 = props.param1;
    this.param2 = props.param2;
  }
  
  static schema = new Map([
    [
      InitializeInstruction,
      {
        kind: 'struct',
        fields: [
          ['param1', 'u64'],
          ['param2', 'u8'],
        ],
      },
    ],
  ]);
}

class Operation1Instruction {
  amount: BN;
  
  constructor(props: { amount: BN }) {
    this.amount = props.amount;
  }
  
  static schema = new Map([
    [
      Operation1Instruction,
      {
        kind: 'struct',
        fields: [
          ['amount', 'u64'],
        ],
      },
    ],
  ]);
}

/**
 * Initialize a new template account
 */
async function initialize(
  connection: Connection,
  authority: Keypair,
  account: Keypair,
  param1: BN,
  param2: number
) {
  // Create initialize instruction
  const initializeInstruction = new InitializeInstruction({
    param1,
    param2,
  });
  
  const instructionData = Buffer.from([TemplateInstructionType.Initialize]);
  const serializedData = borsh.serialize(
    InitializeInstruction.schema,
    initializeInstruction
  );
  
  const fullData = Buffer.concat([instructionData, serializedData]);
  
  const transaction = new Transaction().add({
    keys: [
      { pubkey: authority.publicKey, isSigner: true, isWritable: true },
      { pubkey: account.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: fullData,
  });
  
  await sendAndConfirmTransaction(
    connection,
    transaction,
    [authority, account]
  );
  
  console.log('Account initialized successfully!');
}

/**
 * Main example function
 */
async function main() {
  // Connect to devnet
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  
  // Generate keypairs for testing
  const authority = Keypair.generate();
  const account = Keypair.generate();
  
  console.log('Requesting airdrop for authority...');
  const airdropSignature = await connection.requestAirdrop(authority.publicKey, 1000000000);
  await connection.confirmTransaction(airdropSignature);
  
  // Initialize with example values
  await initialize(
    connection,
    authority,
    account,
    new BN(100),
    5
  );
  
  // TODO: Add examples for other operations
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  }
);
