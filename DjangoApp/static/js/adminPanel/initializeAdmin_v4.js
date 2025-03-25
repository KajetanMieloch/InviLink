async function initializeAdmin() {

    
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;

    await initConnection();
    //Connect to Solana Devnet
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
    //ID of the Anchor program
    //Calculate PDA (Program Derived Address) for `event_registry`
    const [eventRegistry] = await solanaWeb3.PublicKey.findProgramAddress(
        [new TextEncoder().encode("event_registry")],  // Seed compatible with Anchor.toml
        PROGRAM_ID
    );

      //Calculate PDA (Program Derived Address) for `organizers_pool`
    const [organizersPool] = await solanaWeb3.PublicKey.findProgramAddress(
        [new TextEncoder().encode("organizers_pool")],  // Seed compatible with Anchor.toml
        PROGRAM_ID
    );

    console.log("Organizers Pool PDA: " + eventRegistry.toBase58());

    //eventRegistryDiscriminator for the `initialize_event_registry` function
    const eventRegistryDiscriminator = new Uint8Array([222, 221, 108, 11, 214, 161, 6, 121]);
    const organizersPoolDiscriminator = new Uint8Array([213, 153, 51, 23, 150, 192, 71, 166]);

    //Create transaction instruction
    const eventRegistryInstruction = new solanaWeb3.TransactionInstruction({
        keys: [
            { pubkey: eventRegistry, isSigner: false, isWritable: true },  // PDA acc
            { pubkey: walletPublicKey, isSigner: true, isWritable: true },     // Payer
            { pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false }  // System Program
        ],
        programId: PROGRAM_ID,
        data: eventRegistryDiscriminator // We send only 8 bytes from the function eventRegistryDiscriminator
    });

    const organizersPoolInstruction = new solanaWeb3.TransactionInstruction({
        keys: [
            { pubkey: organizersPool, isSigner: false, isWritable: true },  // PDA acc
            { pubkey: walletPublicKey, isSigner: true, isWritable: true },     // Payer
            { pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false }  // System Program
        ],
        programId: PROGRAM_ID,
        data: organizersPoolDiscriminator // We send only 8 bytes from the function eventRegistryDiscriminator
    });

    try {
        //Create transaction
        let transaction = new solanaWeb3.Transaction();
        transaction.add(eventRegistryInstruction);
        transaction.add(organizersPoolInstruction);

        transaction.feePayer = walletPublicKey;
        const { blockhash } = await connection.getLatestBlockhash();
        transaction.recentBlockhash = blockhash;

        //Sign transaction by the user
        const signedTransaction = await provider.signTransaction(transaction);

        //Send transaction
        const txSignature = await connection.sendRawTransaction(signedTransaction.serialize());
        console.log("Transaction sent: " + txSignature);

        //Confirm transaction
        const confirmation = await connection.confirmTransaction(txSignature, "confirmed");
        if (confirmation.value.err) {
            throw new Error("Transaction failed: " + JSON.stringify(confirmation.value.err));
        }

        console.log("Organizers Pool and Event Registry initialized! Tx Sig: " + txSignature);
        showSuccessAlert("Organizers Pool and Event Registry initialized! Tx Sig: " + txSignature);
    } catch (err) {
        console.log("Error: " + err);
        showErrorAlertwithMSG("Error: " + err);
    }
}