import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Invilink } from "../target/types/invilink";
import { Keypair, SystemProgram } from "@solana/web3.js";

describe("Initialize organizers pool", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Invilink as Program<Invilink>;

  // Nowy klucz do puli organizatorów
  const organizersPool = Keypair.generate();

  it("Initializes the organizers pool", async () => {
    const tx = await program.methods
      .initializeOrganizersPool()
      .accounts({
        organizersPool: organizersPool.publicKey,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([organizersPool])
      .rpc();

    console.log("TX Signature:", tx);
  });
});
