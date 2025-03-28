async function fetchOrganizers() {

  const constants = await getConstants();
  const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
  const NETWORK = constants.NETWORK;


  await initConnection();

  const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

  // Calculating PDA for organizers_pool
  const [organizersPoolPDA] = await solanaWeb3.PublicKey.findProgramAddress(
    [new TextEncoder().encode("organizers_pool")],
    PROGRAM_ID
  );

  console.log("Organizers Pool PDA: " + organizersPoolPDA.toBase58());

  try {
    const accountInfo = await connection.getAccountInfo(organizersPoolPDA);
    if (!accountInfo) {
      throw new Error("No organizers data found!");
    }

    // Logging account data
    console.log("Raw account data:", accountInfo.data);

    // Reading the number of organizers (offset by 8 bytes due to discriminator)
    const organizersCount = new DataView(accountInfo.data.buffer).getUint32(8, true);
    console.log("Number of organizers: " + organizersCount);

    // Organizers list
    const organizersList = [];
    const nullPubkey = new solanaWeb3.PublicKey("11111111111111111111111111111111"); // Empty Solana address

    // Reading only valid addresses
    for (let i = 12, count = 0; count < organizersCount; i += 32, count++) {
      const pubkeyBytes = accountInfo.data.slice(i, i + 32);
      const organizerPubkey = new solanaWeb3.PublicKey(pubkeyBytes);

      // Check if the address is not empty (invalid)
      if (!organizerPubkey.equals(nullPubkey)) {
        organizersList.push(organizerPubkey.toBase58());
      }
    }

    console.log("Organizers retrieved: " + JSON.stringify(organizersList));

    renderOrganizersList(organizersList);
  } catch (err) {
    console.log("Error: " + err.message);
  }
}

function renderOrganizersList(organizers) {
  const tbody = document.getElementById("organizersList");
  tbody.innerHTML = "";

  if (organizers.length === 0) {
    tbody.innerHTML = `<tr><td colspan="3">No organizers</td></tr>`;
    return;
  }

  organizers.forEach((organizer, index) => {
    const row = document.createElement("tr");

    row.innerHTML = `
      <td>${index + 1}</td>
      <td>${organizer}</td>
      <td><button onclick="removeOrganizer('${organizer}')">Remove</button></td>
    `;

    tbody.appendChild(row);
  });
}

async function removeOrganizer(organizerAddress) {

    
const constants = await getConstants();
const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
const NETWORK = constants.NETWORK;


  console.log("Removing organizer: " + organizerAddress + "...");

  if (!window.phantom || !window.phantom.solana) {
    showErrorAlertwithMSG("Phantom Wallet is required!");
    return;
  }

  const provider = window.phantom.solana;
  if (!provider.isConnected) {
    await provider.connect();
  }

  const walletPublicKey = provider.publicKey;
  console.log("Your public key: " + walletPublicKey.toBase58());

  const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

  // Calculating PDA for organizers_pool
  const [organizersPoolPDA] = await solanaWeb3.PublicKey.findProgramAddress(
    [new TextEncoder().encode("organizers_pool")],
    PROGRAM_ID
  );

  console.log("Organizers Pool PDA: " + organizersPoolPDA.toBase58());

  // Discriminator for the `remove_organizer` function
  const discriminator = new Uint8Array([64, 187, 72, 87, 252, 241, 195, 60]); // Discriminator from IDL

  function pubkeyToUint8Array(pubkey) {
    return new Uint8Array(new solanaWeb3.PublicKey(pubkey).toBytes());
  }

  const organizerBytes = pubkeyToUint8Array(organizerAddress);

  const instructionData = new Uint8Array([...discriminator, ...organizerBytes]);

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
    console.log("Transaction sent, signature: " + txSignature);

    const confirmation = await connection.confirmTransaction(txSignature, "confirmed");
    if (confirmation.value.err) {
      throw new Error("Transaction failed: " + JSON.stringify(confirmation.value.err));
    }

    console.log("Organizer removed! Tx Sig: " + txSignature);
    showSuccessAlert("Organizer removed! Tx Sig: " + txSignature);

    fetchOrganizers(); // Refresh the list
  } catch (err) {
    console.log("Error: " + err.message);
    showErrorAlert("Error: " + err.message);
  }
}
