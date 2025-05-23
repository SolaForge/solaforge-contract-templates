/**
 * Example client for Single-Token Staking operations
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
import { 
  Token, 
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import BN from 'bn.js';
import * as borsh from 'borsh';

// Define the program ID (replace with your actual program ID)
const PROGRAM_ID = new PublicKey('Stake111111111111111111111111111111111111111');

// Define instruction types
enum StakingInstructionType {
  InitializePool = 0,
  Stake = 1,
  Unstake = 2,
  ClaimRewards = 3,
  UpdatePool = 4,
  FundRewards = 5,
}

// Define instruction schema
class InitializePoolInstruction {
  rewardRate: BN;
  minStakeDuration: BN;
  earlyWithdrawalPenalty: number;
  
  constructor(props: { rewardRate: BN; minStakeDuration: BN; earlyWithdrawalPenalty: number }) {
    this.rewardRate = props.rewardRate;
    this.minStakeDuration = props.minStakeDuration;
    this.earlyWithdrawalPenalty = props.earlyWithdrawalPenalty;
  }
  
  static schema = new Map([
    [
      InitializePoolInstruction,
      {
        kind: 'struct',
        fields: [
          ['rewardRate', 'u64'],
          ['minStakeDuration', 'u64'],
          ['earlyWithdrawalPenalty', 'u16'],
        ],
      },
    ],
  ]);
}

class StakeInstruction {
  amount: BN;
  lockDuration: BN;
  
  constructor(props: { amount: BN; lockDuration: BN }) {
    this.amount = props.amount;
    this.lockDuration = props.lockDuration;
  }
  
  static schema = new Map([
    [
      StakeInstruction,
      {
        kind: 'struct',
        fields: [
          ['amount', 'u64'],
          ['lockDuration', 'u64'],
        ],
      },
    ],
  ]);
}

/**
 * Initialize a staking pool
 */
async function initializePool(
  connection: Connection,
  authority: Keypair,
  stakePoolAccount: Keypair,
  tokenMint: PublicKey,
  poolTokenAccount: PublicKey,
  poolRewardAccount: PublicKey,
  rewardRate: BN,
  minStakeDuration: BN,
  earlyWithdrawalPenalty: number
) {
  // Serialize InitializePool instruction
  const initPoolInstruction = new InitializePoolInstruction({
    rewardRate,
    minStakeDuration,
    earlyWithdrawalPenalty,
  });
  
  const initPoolData = Buffer.from([StakingInstructionType.InitializePool]);
  const instructionData = borsh.serialize(
    InitializePoolInstruction.schema,
    initPoolInstruction
  );
  
  const fullData = Buffer.concat([initPoolData, instructionData]);
  
  const transaction = new Transaction().add({
    keys: [
      { pubkey: authority.publicKey, isSigner: true, isWritable: true },
      { pubkey: stakePoolAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: tokenMint, isSigner: false, isWritable: false },
      { pubkey: poolTokenAccount, isSigner: false, isWritable: true },
      { pubkey: poolRewardAccount, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: fullData,
  });
  
  await sendAndConfirmTransaction(
    connection,
    transaction,
    [authority, stakePoolAccount]
  );
  
  console.log('Staking pool initialized successfully!');
  console.log('Stake pool address:', stakePoolAccount.publicKey.toString());
}

/**
 * Stake tokens
 */
async function stakeTokens(
  connection: Connection,
  user: Keypair,
  stakePoolAccount: PublicKey,
  poolTokenAccount: PublicKey,
  userStakeAccount: Keypair,
  userTokenAccount: PublicKey,
  amount: BN,
  lockDuration: BN
) {
  // Serialize Stake instruction
  const stakeInstruction = new StakeInstruction({
    amount,
    lockDuration,
  });
  
  const stakeData = Buffer.from([StakingInstructionType.Stake]);
  const instructionData = borsh.serialize(
    StakeInstruction.schema,
    stakeInstruction
  );
  
  const fullData = Buffer.concat([stakeData, instructionData]);
  
  const transaction = new Transaction().add({
    keys: [
      { pubkey: user.publicKey, isSigner: true, isWritable: true },
      { pubkey: stakePoolAccount, isSigner: false, isWritable: true },
      { pubkey: poolTokenAccount, isSigner: false, isWritable: true },
      { pubkey: userStakeAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: userTokenAccount, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: fullData,
  });
  
  await sendAndConfirmTransaction(
    connection,
    transaction,
    [user, userStakeAccount]
  );
  
  console.log('Tokens staked successfully!');
  console.log('User stake account:', userStakeAccount.publicKey.toString());
  console.log('Amount:', amount.toString());
}

/**
 * Main example function
 */
async function main() {
  // Connect to devnet
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  
  // Generate new keypairs for testing
  const authority = new Keypair();
  const stakePoolAccount = new Keypair();
  const tokenMint = Keypair.generate();
  
  console.log('Requesting airdrop for authority...');
  const airdropSignature = await connection.requestAirdrop(authority.publicKey, 2 * 10**9);
  await connection.confirmTransaction(airdropSignature);
  
  // Initialize mint, token accounts, etc. (not shown here)
  // Then initialize staking pool with 1% daily reward, 7-day lock, 5% penalty
  
  await initializePool(
    connection,
    authority,
    stakePoolAccount,
    tokenMint.publicKey,
    new PublicKey('PoolTokenAccount111111111111111111111111111'),
    new PublicKey('PoolRewardAccount111111111111111111111111111'),
    new BN(100), // 1% daily
    new BN(86400 * 7), // 7 days
    500 // 5%
  );
  
  // To stake tokens, claim rewards, and unstake, you would follow similar patterns
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  }
);
