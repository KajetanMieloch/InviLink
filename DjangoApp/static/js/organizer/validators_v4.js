async function addValidator() {
    const eventId = window.currentEvent.event_id;
    const validatorStr = document.getElementById("validatorInput").value.trim();
  
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  
    await initConnection();
  
    const ADD_VALIDATOR_DISCRIMINATOR = new Uint8Array([250, 113, 53, 54, 141, 117, 215, 185]);
  
    if (!validatorStr) {
      showErrorAlertwithMSG("Please enter a validator PublicKey.");
      return;
    }
  
    let validatorPubkey;
    try {
      validatorPubkey = new solanaWeb3.PublicKey(validatorStr);
    } catch (err) {
      showErrorAlertwithMSG("Invalid validator PublicKey format!");
      return;
    }
  
    // Compute event PDA: seeds = ["event", eventId]
    const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("event"), new TextEncoder().encode(eventId)],
      PROGRAM_ID
    );
    console.log("Computed Event PDA: " + eventPDA.toBase58());
  
    // Build instruction data: discriminator + 32 bytes of validator pubkey
    const data = new Uint8Array(ADD_VALIDATOR_DISCRIMINATOR.length + 32);
    data.set(ADD_VALIDATOR_DISCRIMINATOR, 0);
    data.set(validatorPubkey.toBytes(), ADD_VALIDATOR_DISCRIMINATOR.length);
  
    const instruction = new solanaWeb3.TransactionInstruction({
      keys: [
        { pubkey: eventPDA, isWritable: true, isSigner: false },
        { pubkey: walletPublicKey, isWritable: false, isSigner: true }
      ],
      programId: PROGRAM_ID,
      data: data
    });
  
    const transaction = new solanaWeb3.Transaction().add(instruction);
    transaction.feePayer = walletPublicKey;
    const { blockhash } = await connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
  
    try {
      const signedTx = await provider.signTransaction(transaction);
      const txSig = await connection.sendRawTransaction(signedTx.serialize());
      await connection.confirmTransaction(txSig, "confirmed");
      showSuccessAlert("Validator added successfully! Tx Signature: " + txSig);
    } catch (err) {
      showErrorAlert("Error adding validator: " + err.message);
      console.error(err);
    }
    window.location.reload();
  }

  async function listValidators() {
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  
    await initConnection();
  
    const eventId = window.currentEvent.event_id;
  
    const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("event"), new TextEncoder().encode(eventId)],
      PROGRAM_ID
    );
    console.log("Computed Event PDA: " + eventPDA.toBase58());
  
    const accountInfo = await connection.getAccountInfo(eventPDA);
    if (!accountInfo) {
      console.error("Event account not found.");
      document.getElementById("validatorTableBody").innerHTML =
        "<tr><td colspan='3'>Event account not found.</td></tr>";
      return;
    }
  
    const eventData = decodeEventWithValidators(accountInfo.data);
    const validators = eventData.validators || [];
  
    const tbody = document.getElementById("validatorTableBody");
    tbody.innerHTML = "";
  
    if (validators.length === 0) {
      tbody.innerHTML = "<tr><td colspan='3'>No validators added yet.</td></tr>";
      return;
    }
  
    validators.forEach((validator, index) => {
      const tr = document.createElement("tr");
      tr.innerHTML = `
        <td>${index + 1}</td>
        <td>${validator}</td>
        <td>
          <button onclick="removeValidator('${validator}')">Remove</button>
        </td>
      `;
      tbody.appendChild(tr);
    });    
  }

  async function removeValidator(pubkeyStr) {
    const eventId = window.currentEvent.event_id;
  
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  
    await initConnection();
  
    const REMOVE_VALIDATOR_DISCRIMINATOR = new Uint8Array([25, 96, 211, 155, 161, 14, 168, 188]);
  
    let validatorPubkey;
    try {
      validatorPubkey = new solanaWeb3.PublicKey(pubkeyStr);
    } catch (err) {
      showErrorAlert("Invalid PublicKey format.");
      return;
    }
  
    const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("event"), new TextEncoder().encode(eventId)],
      PROGRAM_ID
    );
  
    // Build instruction data
    const data = new Uint8Array(REMOVE_VALIDATOR_DISCRIMINATOR.length + 32);
    data.set(REMOVE_VALIDATOR_DISCRIMINATOR, 0);
    data.set(validatorPubkey.toBytes(), REMOVE_VALIDATOR_DISCRIMINATOR.length);
  
    const instruction = new solanaWeb3.TransactionInstruction({
      keys: [
        { pubkey: eventPDA, isWritable: true, isSigner: false },
        { pubkey: walletPublicKey, isWritable: false, isSigner: true }
      ],
      programId: PROGRAM_ID,
      data: data
    });
  
    const transaction = new solanaWeb3.Transaction().add(instruction);
    transaction.feePayer = walletPublicKey;
    const { blockhash } = await connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
  
    try {
      const signedTx = await provider.signTransaction(transaction);
      const txSig = await connection.sendRawTransaction(signedTx.serialize());
      await connection.confirmTransaction(txSig, "confirmed");
      showSuccessAlert("Validator removed! Tx Sig: " + txSig);
      await listValidators(); // Refresh table
    } catch (err) {
      showErrorAlert("Error removing validator: " + err.message);
      console.error(err);
    }
  }
  