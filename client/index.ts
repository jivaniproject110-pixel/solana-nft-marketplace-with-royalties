```typescript
import { web3 } from '@coral-xyz/anchor';
import { Idl, Provider, Program } from '@coral-xyz/anchor';
import { Wallet } from '@coral-xyz/anchor/dist/cjs/provider';
import { Keypair, PublicKey } from '@solana/web3.js';

// Set up wallet adapter
const wallet = {
  // Replace with your wallet's publicKey and signTransaction function
  publicKey: new PublicKey('your-public-key'),
  signTransaction: async (transaction: web3.Transaction) => {
    // Implement your wallet's signTransaction function
    // For example, using @solana/wallet-adapter-react:
    // const { signTransaction } = useWallet();
    // return await signTransaction(transaction);
  },
  signAllTransactions: async (transactions: web3.Transaction[]) => {
    // Implement your wallet's signAllTransactions function
    // For example, using @solana/wallet-adapter-react:
    // const { signAllTransactions } = useWallet();
    // return await signAllTransactions(transactions);
  },
};

// Set up connection to Solana cluster
const connection = new web3.Connection('https://api.devnet.solana.com');

// Set up provider
const provider = new Provider(connection, wallet as Wallet, {
  // Replace with your preferred commitment level
  commitment: web3.Commitment.Confirmed,
});

// Import IDL
import idlJson from './idl.json';
const idl: Idl = idlJson;

// Create program instance
const program = new Program(idl, 'your-program-id', provider);

// Create a new NFT
async function createNFT() {
  // Generate a new Keypair for the NFT
  const nftKeypair = Keypair.generate();

  // Call the createNFT instruction
  const tx = await program.methods
    .createNFT()
    .accounts({
      // Replace with the correct accounts
      nft: nftKeypair.publicKey,
      authority: wallet.publicKey,
      systemProgram: web3.SystemProgram.programId,
    })
    .rpc();
}

// Fetch an NFT account
async function fetchNFT(publicKey: PublicKey) {
  // Fetch the NFT account
  const nftAccount = await program.account.nft.fetch(publicKey);
  return nftAccount;
}

// Update an NFT's royalty
async function updateRoyalty(publicKey: PublicKey, royalty: number) {
  // Call the updateRoyalty instruction
  const tx = await program.methods
    .updateRoyalty(royalty)
    .accounts({
      // Replace with the correct accounts
      nft: publicKey,
      authority: wallet.publicKey,
    })
    .rpc();
}

createNFT();
fetchNFT(new PublicKey('nft-public-key'));
updateRoyalty(new PublicKey('nft-public-key'), 10);
```

**Note:** 

1. Replace `'your-program-id'` with the actual ID of your Solana program.
2. Replace `idl.json` with the path to your IDL file.
3. Replace `'your-public-key'` with the actual public key of your wallet.
4. Implement the `signTransaction` and `signAllTransactions` functions according to your wallet's documentation.
5. This code assumes you are using the `@coral-xyz/anchor` version `0.29.0`. Make sure to check the documentation for the correct syntax and usage.

**Example Use Cases:**

1. Creating a new NFT: Call the `createNFT` function to create a new NFT.
2. Fetching an NFT account: Call the `fetchNFT` function with the public key of the NFT to fetch its account.
3. Updating an NFT's royalty: Call the `updateRoyalty` function with the public key of the NFT and the new royalty to update the royalty.