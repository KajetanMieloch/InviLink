let scannerInstance = null;

function parseQRParams(qrData) {
  const params = {};
  qrData.split("&").forEach(pair => {
    const [key, value] = pair.split("=");
    if (key && value) {
      const decoded = decodeURIComponent(value).replace(/!\(_\)!/g, " ");
      params[key] = decoded;
    }
  });
  return params;
}

async function validateTicketFromQR(decodedText) {
  try {
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

    await initConnection();

    const params = parseQRParams(decodedText);
    const eventId = params.eventId;
    const section = params.section;
    const row = parseInt(params.row);
    const seat = parseInt(params.seat);

    if (!eventId || !section || isNaN(row) || isNaN(seat)) {
      showErrorAlertwithMSG("Invalid QR code content.");
      return;
    }

    const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("event"), new TextEncoder().encode(eventId)],
      PROGRAM_ID
    );

    const [ticketStatusPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [
        new TextEncoder().encode("ticket_status"),
        new TextEncoder().encode(eventId),
        new TextEncoder().encode(section),
        new Uint8Array([row]),
        new Uint8Array([seat])
      ],
      PROGRAM_ID
    );

    const data = buildValidateTicketData(eventId, section, row, seat);
    const keys = [
      { pubkey: eventPDA, isWritable: true, isSigner: false },
      { pubkey: ticketStatusPDA, isWritable: true, isSigner: false },
      { pubkey: walletPublicKey, isWritable: false, isSigner: true }
    ];

    const instruction = new solanaWeb3.TransactionInstruction({
      keys,
      programId: PROGRAM_ID,
      data
    });

    const transaction = new solanaWeb3.Transaction().add(instruction);
    transaction.feePayer = walletPublicKey;
    const { blockhash } = await connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;

    const signedTx = await provider.signTransaction(transaction);
    const txSig = await connection.sendRawTransaction(signedTx.serialize());
    await connection.confirmTransaction(txSig, "confirmed");

    showSuccessAlert("Ticket validated successfully!");
  } catch (err) {
    console.error(err);
    showErrorAlert("Error: " + err.message);
  }
}

function startScanner() {
  if (scannerInstance) {
    scannerInstance.clear().then(() => {
      scannerInstance.render(onScanSuccess);
    });
  } else {
    scannerInstance = new Html5QrcodeScanner("reader", { fps: 10, qrbox: 250 });
    scannerInstance.render(onScanSuccess);
  }
}

async function onScanSuccess(decodedText, decodedResult) {
  console.log("QR Scanned: " + decodedText);
  if (scannerInstance) {
    await scannerInstance.clear();
  }
  await validateTicketFromQR(decodedText);
}

window.onload = () => {
  startScanner();
};
