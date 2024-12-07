import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { HelloContract } from "../target/types/hello_contract";

describe("hello_contract", () => {
  // Configure the client to use the devnet cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.HelloContract as Program<HelloContract>;

  it("Says hello!", async () => {
    const tx = await program.methods.sayHello().rpc();
    console.log("Transaction signature for sayHello:", tx);
  });

  it("Creates PDA!", async () => {
    const [pda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("my-pda")],
      program.programId
    );

    console.log("Program ID:", program.programId.toBase58());
    console.log("PDA:", pda.toBase58());
    console.log("Bump:", bump);

    const tx = await program.methods
      .createPda()
      .accounts({
        pda: pda,
        user: program.provider.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Transaction signature for PDA creation:", tx);
  });
});
