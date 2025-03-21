async function addOrganizer() {

    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;

    if (!window.phantom || !window.phantom.solana) {
      alert("Phantom Wallet is required!");
      return;
    }

    const provider = window.phantom.solana;
    if (!provider.isConnected) {
      await provider.connect();
    }

    const walletPublicKey = provider.publicKey;
    logMessage("Your public key: " + walletPublicKey.toBase58());

    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

    const newOrganizerAddress = document.getElementById("organizerAddress").value.trim();
    if (!newOrganizerAddress) {
      alert("Please provide a valid organizer public address.");
      return;
    }

    let newOrganizerPubkey;
    try {
      newOrganizerPubkey = new solanaWeb3.PublicKey(newOrganizerAddress);
    } catch (err) {
      alert("Invalid public key format.");
      return;
    }

    // PDA for `organizers_pool`
    const [organizersPoolPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("organizers_pool")], 
      PROGRAM_ID
    );

    logMessage("Organizers Pool PDA: " + organizersPoolPDA.toBase58());

    // Discriminator for the `add_organizer` function
    const discriminator = new Uint8Array([142, 52, 252, 155, 155, 95, 29, 215]);

    // Converting public key to Uint8Array
    function pubkeyToUint8Array(pubkey) {
      return new Uint8Array(pubkey.toBytes());
    }

    const newOrganizerBytes = pubkeyToUint8Array(newOrganizerPubkey);

    // Creating transaction instruction (without Buffer)
    const instructionData = new Uint8Array([...discriminator, ...newOrganizerBytes]);

    const transactionInstruction = new solanaWeb3.TransactionInstruction({
      keys: [
        { pubkey: organizersPoolPDA, isSigner: false, isWritable: true },
        { pubkey: walletPublicKey, isSigner: true, isWritable: true }
      ],
      programId: PROGRAM_ID,
      data: instructionData
    });

    try {
      let transaction = new solanaWeb3.Transaction().add(transactionInstruction);
      transaction.feePayer = walletPublicKey;
      const { blockhash } = await connection.getLatestBlockhash();
      transaction.recentBlockhash = blockhash;

      const signedTransaction = await provider.signTransaction(transaction);
      const txSignature = await connection.sendRawTransaction(signedTransaction.serialize());
      logMessage("Transaction sent, signature: " + txSignature);

      const confirmation = await connection.confirmTransaction(txSignature, "confirmed");
      if (confirmation.value.err) {
        throw new Error("Transaction failed: " + JSON.stringify(confirmation.value.err));
      }

      logMessage("Organizer added! Tx Sig: " + txSignature);
    } catch (err) {
      logMessage("Error: " + err.message);
    }
}
