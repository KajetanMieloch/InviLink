import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Invilink } from "../target/types/invilink";

describe("remove_organizer", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Invilink as Program<Invilink>;

  const [organizersPoolPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("organizers_pool")],
    program.programId
  );
  const organizerToRemove = anchor.web3.Keypair.generate();
  before("Add organizer for test", async () => {
    await program.methods
      .addOrganizer(organizerToRemove.publicKey)
      .accounts({
        organizersPool: organizersPoolPda,
        signer: provider.wallet.publicKey,
      })
      .rpc();
  });

  it("Usuwa organizatora z puli", async () => {
    await program.methods
      .removeOrganizer(organizerToRemove.publicKey)
      .accounts({
        organizersPool: organizersPoolPda,
        signer: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Usunięto organizatora:", organizerToRemove.publicKey.toBase58());
  });
});
