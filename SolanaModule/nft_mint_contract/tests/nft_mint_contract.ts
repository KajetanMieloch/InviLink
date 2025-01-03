import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { NftMintContract } from "../target/types/nft_mint_contract";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  createAccount,
} from "@solana/spl-token";
import { Keypair } from "@solana/web3.js";
import assert from "assert";

describe("nft_mint_contract", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.NftMintContract as Program<NftMintContract>;

  let mint: anchor.web3.PublicKey;
  let payerTokenAccount: anchor.web3.PublicKey;
  let recipientTokenAccount: anchor.web3.PublicKey;

  it("Mints an NFT", async () => {
    // Create a new mint
    mint = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null, // Freeze authority (optional for NFTs)
      0 // Decimals (0 for NFTs)
    );

    // Create a token account for the payer
    payerTokenAccount = await createAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      provider.wallet.publicKey
    );

    // Call the mint_token instruction
    await program.methods
      .mintToken(new anchor.BN(1)) // Mint 1 token (NFT)
      .accounts({
        mint,
        to: payerTokenAccount,
        payer: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Check the token account's balance
    const payerAccountInfo = await getAccount(provider.connection, payerTokenAccount);
    assert.strictEqual(Number(payerAccountInfo.amount), 1, "Minted amount should be 1");
  });

  it("Transfers the NFT", async () => {
    // Create a token account for the recipient
    const recipient = Keypair.generate().publicKey;
    recipientTokenAccount = await createAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      recipient
    );

    // Call the transfer_token instruction
    await program.methods
      .transferToken(new anchor.BN(1)) // Transfer 1 token (NFT)
      .accounts({
        from: payerTokenAccount,
        to: recipientTokenAccount,
        signer: provider.wallet.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Check the token accounts' balances
    const payerAccountInfo = await getAccount(provider.connection, payerTokenAccount);
    const recipientAccountInfo = await getAccount(provider.connection, recipientTokenAccount);

    assert.strictEqual(Number(payerAccountInfo.amount), 0, "Payer balance should be 0");
    assert.strictEqual(Number(recipientAccountInfo.amount), 1, "Recipient balance should be 1");
  });
});
