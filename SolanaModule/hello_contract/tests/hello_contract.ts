import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { HelloContract } from "../target/types/hello_contract";

describe("hello_contract", () => {
  // Configure the client to use the devnet cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.HelloContract as Program<HelloContract>;

  it("Says hello!", async () => {
    // Call the say_hello method of the program
    const tx = await program.methods.sayHello().rpc();
    console.log("Your transaction signature", tx);
  });
});
