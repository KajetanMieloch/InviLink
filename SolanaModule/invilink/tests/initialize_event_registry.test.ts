import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Invilink } from "../target/types/invilink";

describe("initialize_event_registry", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Invilink as Program<Invilink>;

  const [eventRegistryPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("event_registry")],
    program.programId
  );

  it("Inicjalizuje rejestr eventów", async () => {
    await program.methods
      .initializeEventRegistry()
      .accounts({
        registry: eventRegistryPda,
        payer: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Rejestr eventów zainicjalizowany:", eventRegistryPda.toBase58());
  });
});
