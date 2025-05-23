/**
 * Example client for NFT Marketplace operations
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
  AccountLayout, 
  Token, 
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import BN from 'bn.js';
import * as borsh from 'borsh';

// Define the program ID (replace with your actual program ID)
const PROGRAM_ID = new PublicKey('NFTMarket111111111111111111111111111111111111');

// Define instruction types
enum MarketplaceInstructionType {
  InitializeMarketplace = 0,
  ListNFT = 1,
  BuyNFT = 2,
  CancelListing = 3,
  UpdateMarketplaceFees = 4,
}

// Define instruction schema
class InitializeMarketplaceInstruction {
  feeBasisPoints: number;
  
  constructor(props: { feeBasisPoints: number }) {
    this.feeBasisPoints = props.feeBasisPoints;
  }
  
  static schema = new Map([
    [
      InitializeMarketplaceInstruction,
      {
        kind: 'struct',
        fields: [
          ['feeBasisPoints', 'u16'],
        ],
      },
    ],
  ]);
}

class ListNFTInstruction {
  price: BN;
  
  constructor(props: { price: BN }) {
    this.price = props.price;
  }
  
  static schema = new Map([
    [
      ListNFTInstruction,
      {
        kind: 'struct',
        fields: [
          ['price', 'u64'],
        ],
      },
    ],
  ]);
}

/**
 * Initialize a marketplace
 */
async function initializeMarketplace(
  connection: Connection,
  authority: Keypair,
  marketplaceAccount: Keypair,
  treasuryAccount: PublicKey,
  feeBasisPoints: number
) {
  // Serialize InitializeMarketplace instruction
  const initMarketplaceInstruction = new InitializeMarketplaceInstruction({
    feeBasisPoints,
  });
  
  const initMarketplaceData = Buffer.from([MarketplaceInstructionType.InitializeMarketplace]);
  const instructionData = borsh.serialize(
    InitializeMarketplaceInstruction.schema,
    initMarketplaceInstruction
  );
  
  const fullData = Buffer.concat([initMarketplaceData, instructionData]);
  
  const transaction = new Transaction().add({
    keys: [
      { pubkey: authority.publicKey, isSigner: true, isWritable: true },
      { pubkey: marketplaceAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: treasuryAccount, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data: fullData,
  });
  
  await sendAndConfirmTransaction(
    connection,
    transaction,
    [authority, marketplaceAccount]
  );
  
  console.log('Marketplace initialized successfully!');
  console.log('Marketplace address:', marketplaceAccount.publicKey.toString());
  console.log('Treasury address:', treasuryAccount.toString());
}

/**
 * List an NFT for sale
 */
async function listNFT(
  connection: Connection,
  seller: Keypair,
  listingAccount: Keypair,
  nftMint: PublicKey,
  sellerTokenAccount: PublicKey,
  marketplaceAccount: PublicKey,
  price: BN
) {
  // Serialize ListNFT instruction
  const listNFTInstruction = new ListNFTInstruction({
    price,
  });
  
  const listNFTData = Buffer.from([MarketplaceInstructionType.ListNFT]);
  const instructionData = borsh.serialize(
    ListNFTInstruction.schema,
    listNFTInstruction
  );
  
  const fullData = Buffer.concat([listNFTData, instructionData]);
  
  const transaction = new Transaction().add({
    keys: [
      { pubkey: seller.publicKey, isSigner: true, isWritable: true },
      { pubkey: listingAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: nftMint, isSigner: false, isWritable: false },
      { pubkey: sellerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: marketplaceAccount, isSigner: false, isWritable: false },
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
    [seller, listingAccount]
  );
  
  console.log('NFT listed successfully!');
  console.log('Listing address:', listingAccount.publicKey.toString());
  console.log('Price:', price.toString());
}

/**
 * Main example function
 */
async function main() {
  // Connect to devnet
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  
  // Generate new keypairs for testing
  const authority = new Keypair();
  const marketplaceAccount = new Keypair();
  const treasuryAccount = new Keypair().publicKey;
  
  console.log('Requesting airdrop for authority...');
  const airdropSignature = await connection.requestAirdrop(authority.publicKey, 2 * 10**9);
  await connection.confirmTransaction(airdropSignature);
  
  // Initialize marketplace with 2.5% fee
  await initializeMarketplace(
    connection,
    authority,
    marketplaceAccount,
    treasuryAccount,
    250 // 2.5%
  );
  
  // To list an NFT, mint a token, and buy or cancel listings, you would follow similar patterns
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  }
);
