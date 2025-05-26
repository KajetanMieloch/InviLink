import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Invilink } from "../target/types/invilink";

describe("add_organizer", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Invilink as Program<Invilink>;

  const [organizersPoolPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("organizers_pool")],
    program.programId
  );

  const newOrganizer = anchor.web3.Keypair.generate();

  it("Dodaje nowego organizatora do puli", async () => {
    await program.methods
      .addOrganizer(newOrganizer.publicKey)
      .accounts({
        organizersPool: organizersPoolPda,
        signer: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Dodano organizatora:", newOrganizer.publicKey.toBase58());
  });
});
