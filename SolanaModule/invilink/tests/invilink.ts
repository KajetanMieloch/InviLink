import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";
import { Invilink } from "../target/types/invilink";

const buyerKeypair = anchor.web3.Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(require("fs").readFileSync("/home/alternator/.config/solana/new_keypair.json", "utf-8")))
);


describe("invilink", () => {
  const provider = new anchor.AnchorProvider(
    new anchor.web3.Connection("https://api.devnet.solana.com", "confirmed"),
    anchor.Wallet.local(),
    {}
  );
  anchor.setProvider(provider);

  const program = anchor.workspace.Invilink as Program<Invilink>;

  let ticketAccount: anchor.web3.Keypair;
  let feePoolAccount: anchor.web3.Keypair;
  const ticketOwner = provider.wallet.publicKey;
  const buyer = anchor.web3.Keypair.generate();
  const feePoolOwner = anchor.web3.Keypair.generate();

  beforeEach(async () => {
    feePoolAccount = anchor.web3.Keypair.generate();
    console.log("Initializing FeePool:");
    console.log("FeePoolAccount PublicKey:", feePoolAccount.publicKey.toBase58());
    console.log("FeePoolOwner PublicKey:", feePoolOwner.publicKey.toBase58());

    await program.methods
      .initializeFeePool(feePoolOwner.publicKey)
      .accounts({
        feePool: feePoolAccount.publicKey,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([feePoolAccount])
      .rpc();
  });

  it("mints a ticket", async () => {
    ticketAccount = anchor.web3.Keypair.generate();

    const eventId = "event123";
    const ticketId = "ticket123";
    const price = new anchor.BN(100);
    const attributes = "Row A, Seat 1";

    console.log("Minting Ticket:");
    console.log("TicketAccount PublicKey:", ticketAccount.publicKey.toBase58());
    console.log("TicketOwner PublicKey:", ticketOwner.toBase58());

    await program.methods
      .mintTicket(eventId, ticketId, price, attributes)
      .accounts({
        ticket: ticketAccount.publicKey,
        owner: ticketOwner,
        systemProgram: SystemProgram.programId,
      })
      .signers([ticketAccount])
      .rpc();

    const ticket = await program.account.ticketNft.fetch(ticketAccount.publicKey);
    console.log("Minted Ticket Details:", ticket);

    expect(ticket.owner.toBase58()).to.equal(ticketOwner.toBase58());
    expect(ticket.eventId).to.equal(eventId);
    expect(ticket.ticketId).to.equal(ticketId);
    expect(ticket.price.toNumber()).to.equal(100);
    expect(ticket.attributes).to.equal(attributes);
    expect(ticket.used).to.equal(false);
  });

  it("sells a ticket", async () => {
    const salePrice = new anchor.BN(120);
  
    console.log("Adding funds to buyer:");
    console.log("Buyer PublicKey:", buyerKeypair.publicKey.toBase58());
  
    // Dodaj fundusze dla Buyer
    const tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: buyerKeypair.publicKey,
        lamports: 1_000_000_000, // 1 SOL
      })
    );
    await provider.sendAndConfirm(tx, []);
  
    console.log("Attempting to sell ticket:");
    console.log("TicketAccount PublicKey:", ticketAccount.publicKey.toBase58());
    console.log("FeePoolAccount PublicKey:", feePoolAccount.publicKey.toBase58());
  
    await program.methods
      .sellTicket(salePrice)
      .accounts({
        ticket: ticketAccount.publicKey,
        seller: ticketOwner,
        buyer: buyerKeypair.publicKey, // Użycie buyerKeypair
        feePool: feePoolAccount.publicKey,
      })
      .signers([buyerKeypair]) // BuyerKeypair jako signer
      .rpc();
  
    const ticket = await program.account.ticketNft.fetch(ticketAccount.publicKey);
    console.log("Ticket After Sale:", ticket);
  
    const feePool = await program.account.feePool.fetch(feePoolAccount.publicKey);
    console.log("FeePool After Sale:", feePool);
  
    expect(ticket.owner.toBase58()).to.equal(buyerKeypair.publicKey.toBase58());
    expect(ticket.price.toNumber()).to.equal(120);
    expect(feePool.totalFees.toNumber()).to.equal(6); // 5% fee
  });
  

  it("transfers a ticket", async () => {
    const newOwner = anchor.web3.Keypair.generate().publicKey;

    console.log("Attempting to transfer ticket:");
    console.log("TicketAccount PublicKey:", ticketAccount.publicKey.toBase58());
    console.log("NewOwner PublicKey:", newOwner.toBase58());

    await program.methods
      .transferTicket(newOwner)
      .accounts({
        ticket: ticketAccount.publicKey,
        currentOwner: buyer.publicKey,
        feePool: feePoolAccount.publicKey,
      })
      .signers([buyer]) // Buyer podpisuje transakcję jako obecny właściciel
      .rpc();

    const ticket = await program.account.ticketNft.fetch(ticketAccount.publicKey);
    console.log("Ticket After Transfer:", ticket);

    expect(ticket.owner.toBase58()).to.equal(newOwner.toBase58());
  });

  it("validates a ticket", async () => {
    const ticketId = "ticket123";

    console.log("Validating ticket:");
    console.log("TicketAccount PublicKey:", ticketAccount.publicKey.toBase58());

    await program.methods
      .validateTicket(ticketId)
      .accounts({
        ticket: ticketAccount.publicKey,
        owner: ticketOwner,
      })
      .rpc();

    console.log("Ticket validated successfully.");
  });

  it("marks a ticket as used", async () => {
    const ticketId = "ticket123";

    console.log("Marking ticket as used:");
    console.log("TicketAccount PublicKey:", ticketAccount.publicKey.toBase58());

    await program.methods
      .markTicketUsed(ticketId)
      .accounts({
        ticket: ticketAccount.publicKey,
        owner: ticketOwner,
      })
      .rpc();

    const ticket = await program.account.ticketNft.fetch(ticketAccount.publicKey);
    console.log("Ticket After Marking Used:", ticket);

    expect(ticket.used).to.equal(true);
  });

  it("withdraws fees", async () => {
    console.log("Withdrawing fees:");
    console.log("FeePoolAccount PublicKey:", feePoolAccount.publicKey.toBase58());

    await program.methods
      .withdrawFees()
      .accounts({
        feePool: feePoolAccount.publicKey,
        owner: feePoolOwner.publicKey,
      })
      .signers([feePoolOwner])
      .rpc();

    const feePool = await program.account.feePool.fetch(feePoolAccount.publicKey);
    console.log("FeePool After Withdrawal:", feePool);

    expect(feePool.totalFees.toNumber()).to.equal(0);
  });
});
