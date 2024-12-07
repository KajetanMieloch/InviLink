import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { HelloContract } from "../target/types/hello_contract";

describe("hello_contract", () => {
  // Configure the client to use the devnet cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.HelloContract as Program<HelloContract>;

  it("Handles payment!", async () => {
    const tx = await program.methods
      .handlePayment()
      .accounts({
        user: program.provider.publicKey, // Adres użytkownika wysyłającego SOL
        systemProgram: anchor.web3.SystemProgram.programId, // Program systemowy
      })
      .rpc();

    console.log("Transaction signature for handle_payment:", tx);
  });
});
