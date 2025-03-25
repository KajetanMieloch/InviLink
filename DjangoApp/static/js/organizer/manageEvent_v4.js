async function updateEvent() {
    const eventPubkey = getEventPubKey();
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  
    const UPDATE_EVENT_DISCRIMINATOR = new Uint8Array([70, 108, 211, 125, 171, 176, 25, 217]);
  
    console.log("Updating event: " + eventPubkey.toBase58());
    const newName = prompt("Enter new event name (leave empty to keep current):");
    const newDate = prompt("Enter new event date (leave empty to keep current):");
    const newAvailableTickets = prompt("Enter new available tickets (leave empty to keep current):");
  
    const encodedName = encodeOptionString(newName);
    const encodedDate = encodeOptionU64(newDate);
    const encodedAvailable = encodeOptionU64(newAvailableTickets);
  
    const totalLength = UPDATE_EVENT_DISCRIMINATOR.length + encodedName.length + encodedDate.length + encodedAvailable.length;
    const updateData = new Uint8Array(totalLength);
    let offset = 0;
    updateData.set(UPDATE_EVENT_DISCRIMINATOR, offset); offset += UPDATE_EVENT_DISCRIMINATOR.length;
    updateData.set(encodedName, offset); offset += encodedName.length;
    updateData.set(encodedDate, offset); offset += encodedDate.length;
    updateData.set(encodedAvailable, offset);
  
    const instruction = new solanaWeb3.TransactionInstruction({
      keys: [
        { pubkey: eventPubkey, isWritable: true, isSigner: false },
        { pubkey: walletPublicKey, isWritable: true, isSigner: true }
      ],
      programId: PROGRAM_ID,
      data: updateData
    });
  
    console.log("update_event instruction created.");
    const transaction = new solanaWeb3.Transaction().add(instruction);
    transaction.feePayer = walletPublicKey;
    const { blockhash } = await connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    console.log("Transaction prepared. Signing...");
    const signedTx = await provider.signTransaction(transaction);
    console.log("Transaction signed.");
    const txSig = await connection.sendRawTransaction(signedTx.serialize());
    console.log("Transaction sent. Signature: " + txSig);
    console.log("Waiting for confirmation...");
    await connection.confirmTransaction(txSig, "confirmed");
    console.log("Transaction confirmed.");
    alert("Event updated! Tx Sig: " + txSig);
    location.reload();
  }
  
  async function activateEvent()
  {
    const eventPubkey = getEventPubKey();
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  
    const ACTIVATE_EVENT_DISCRIMINATOR = new Uint8Array([231, 184, 218, 110, 194, 0, 39, 115]);
  
    console.log("Activating event: " + eventPubkey.toBase58());
    const instruction = new solanaWeb3.TransactionInstruction({
      keys: [
        { pubkey: eventPubkey, isWritable: true, isSigner: false },
        { pubkey: walletPublicKey, isWritable: true, isSigner: true }
      ],
      programId: PROGRAM_ID,
      data: ACTIVATE_EVENT_DISCRIMINATOR
    });
  
    console.log("activate_event instruction created.");
    const transaction = new solanaWeb3.Transaction().add(instruction);
    transaction.feePayer = walletPublicKey;
    const { blockhash } = await connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    console.log("Transaction prepared. Signing...");
    const signedTx = await provider.signTransaction(transaction);
    console.log("Transaction signed.");
    const txSig = await connection.sendRawTransaction(signedTx.serialize());
    console.log("Transaction sent. Signature: " + txSig);
    console.log("Waiting for confirmation...");
    await connection.confirmTransaction(txSig, "confirmed");
    console.log("Transaction confirmed.");
    alert("Event activated! Tx Sig: " + txSig);
    location.reload();
  }
  
  async function deactivateEvent() {
    const eventPubkey = getEventPubKey();
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  
    const DEACTIVATE_EVENT_DISCRIMINATOR = new Uint8Array([222, 84, 182, 86, 46, 110, 215, 19]);
  
    console.log("Deactivating event: " + eventPubkey.toBase58());
    const instruction = new solanaWeb3.TransactionInstruction({
      keys: [
        { pubkey: eventPubkey, isWritable: true, isSigner: false },
        { pubkey: walletPublicKey, isWritable: true, isSigner: true }
      ],
      programId: PROGRAM_ID,
      data: DEACTIVATE_EVENT_DISCRIMINATOR
    });
  
    console.log("deactivate_event instruction created.");
    const transaction = new solanaWeb3.Transaction().add(instruction);
    transaction.feePayer = walletPublicKey;
    const { blockhash } = await connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    console.log("Transaction prepared. Signing...");
    const signedTx = await provider.signTransaction(transaction);
    console.log("Transaction signed.");
    const txSig = await connection.sendRawTransaction(signedTx.serialize());
    console.log("Transaction sent. Signature: " + txSig);
    console.log("Waiting for confirmation...");
    await connection.confirmTransaction(txSig, "confirmed");
    console.log("Transaction confirmed.");
    alert("Event deactivated! Tx Sig: " + txSig);
    location.reload();
  }
  
  async function deleteEvent() {
    const eventPubkey = getEventPubKey();
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  
    const DELETE_EVENT_DISCRIMINATOR = new Uint8Array([103, 111, 95, 106, 232, 24, 190, 84]);
  
    console.log("Deleting event: " + eventPubkey.toBase58());
    const [registryPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode(REGISTRY_SEED)],
      PROGRAM_ID
    );
    console.log("Registry PDA: " + registryPDA.toBase58());
  
    const instruction = new solanaWeb3.TransactionInstruction({
      keys: [
        { pubkey: eventPubkey, isWritable: true, isSigner: false },
        { pubkey: registryPDA, isWritable: true, isSigner: false },
        { pubkey: walletPublicKey, isWritable: true, isSigner: true }
      ],
      programId: PROGRAM_ID,
      data: DELETE_EVENT_DISCRIMINATOR
    });
  
    console.log("delete_event instruction created.");
    const transaction = new solanaWeb3.Transaction().add(instruction);
    transaction.feePayer = walletPublicKey;
    const { blockhash } = await connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    console.log("Transaction prepared. Signing...");
    const signedTx = await provider.signTransaction(transaction);
    console.log("Transaction signed.");
    const txSig = await connection.sendRawTransaction(signedTx.serialize());
    console.log("Transaction sent. Signature: " + txSig);
    console.log("Waiting for confirmation...");
    await connection.confirmTransaction(txSig, "confirmed");
    console.log("Transaction confirmed.");
    alert("Event deleted! Tx Sig: " + txSig);
    window.location.href = "/";
  }

 async function getEventPubKey(){
  const eventId = window.currentEvent.event_id;
  const constants = await getConstants();
  const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
  const NETWORK = constants.NETWORK;
  const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  await initConnection();


  const [registryPDA] = await solanaWeb3.PublicKey.findProgramAddress(
    [new TextEncoder().encode("event_registry")],
    PROGRAM_ID
  );
  
  const regAccount = await connection.getAccountInfo(registryPDA);
  const registry = decodeRegistry(regAccount.data);

  for (let pubkeyStr of registry.events) {
    const eventPubkey = new solanaWeb3.PublicKey(pubkeyStr);
    const eventAcc = await connection.getAccountInfo(eventPubkey);

    if (eventAcc) {
      const eventData = decodeEvent(eventAcc.data);
      if(eventData.event_id === eventId){
        console.log("Event found: " + eventPubkey.toBase58());
        return eventPubkey;
      }
    }
  }

 }