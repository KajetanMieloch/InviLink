let walletPublicKey = null;
const TOKEN_PROGRAM_ID = new solanaWeb3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const METADATA_PROGRAM_ID = new solanaWeb3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

// Funkcja poprawiająca URI z IPFS
function fixIpfsUri(uri) {
  if (!uri) return "";
  uri = uri.trim();
  if (uri.charAt(0) === "�") {
    uri = uri.substring(1);
  }
  if (uri.startsWith("ipfs://")) {
    return uri.replace("ipfs://", "https://ipfs.io/ipfs/");
  }
  return uri;
}

async function loadNFTs() {

  const constants = await getConstants();
  const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
  const NETWORK = constants.NETWORK;
  const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  await initConnection();

  const tokenAccounts = await connection.getParsedTokenAccountsByOwner(walletPublicKey, { programId: TOKEN_PROGRAM_ID });
  console.log("Znaleziono " + tokenAccounts.value.length + " token accounts.");
  const nftAccounts = tokenAccounts.value.filter(acc => {
    const info = acc.account.data.parsed.info;
    return info.tokenAmount.decimals === 0 && info.tokenAmount.uiAmount === 1;
  });
  console.log("Znaleziono " + nftAccounts.length + " NFT.");
  const nftContainer = document.getElementById("nftContainer");
  nftContainer.innerHTML = "";

  for (const acc of nftAccounts) {
    const mintAddress = acc.account.data.parsed.info.mint;
    // Obliczamy PDA dla metadanych wg standardu Metaplex:
    const seed1 = new TextEncoder().encode("metadata");
    const seed2 = METADATA_PROGRAM_ID.toBytes();
    const seed3 = new solanaWeb3.PublicKey(mintAddress).toBytes();
    const [metadataPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [seed1, seed2, seed3],
      METADATA_PROGRAM_ID
    );
    console.log("Mint: " + mintAddress + " | Metadata PDA: " + metadataPDA.toBase58());
    const metadataAccount = await connection.getAccountInfo(metadataPDA);
    if (!metadataAccount) {
      console.log("Brak metadanych dla mint: " + mintAddress);
      continue;
    }
    if (metadataAccount.data.byteLength < 315) {
      console.log("Niewystarczająca długość danych dla mint: " + mintAddress);
      continue;
    }
    const metadata = customDeserializeMetadata(metadataAccount.data);
    console.log("Odczytano metadane: " + JSON.stringify(metadata));

    let metadataJSON = {};
    try {
      const response = await fetch(fixIpfsUri(metadata.uri));
      metadataJSON = await response.json();
    } catch (err) {
      console.log("Błąd pobierania JSON dla mint: " + mintAddress);
    }

    // Wyciągamy dane z JSON
    // Zakładamy, że name ma format "InviLink Ticket - <eventId>"
    let eventIdFromName = "";
    if (metadataJSON.name) {
      const parts = metadataJSON.name.split(" - ");
      if (parts.length >= 2) {
        eventIdFromName = parts[1].trim();
      }
    }
    // Wyciągamy section, row i seat z atrybutów
    let sectionValue = "";
    let rowValue = 0;
    let seatValue = 0;
    if (metadataJSON.attributes && Array.isArray(metadataJSON.attributes)) {
      for (const attr of metadataJSON.attributes) {
        if (attr.trait_type.toLowerCase() === "section") {
          sectionValue = attr.value;
        } else if (attr.trait_type.toLowerCase() === "row") {
          rowValue = parseInt(attr.value);
        } else if (attr.trait_type.toLowerCase() === "seat") {
          seatValue = parseInt(attr.value);
        }
      }
    }

    // Budujemy element NFT z obrazkiem i metadanymi
    const nftDiv = document.createElement("div");
    nftDiv.className = "nft";
    const imageUrl = fixIpfsUri(metadataJSON.image || "");
    let attributesHtml = "";
    if (metadataJSON.attributes && Array.isArray(metadataJSON.attributes)) {
      attributesHtml = "<ul class='attributes'>";
      for (const attr of metadataJSON.attributes) {
        attributesHtml += `<li><strong>${attr.trait_type}:</strong> ${attr.value}</li>`;
      }
      attributesHtml += "</ul>";
    }

    nftDiv.className = "card";
    nftDiv.innerHTML = `
      <img src="${imageUrl}" alt="${metadataJSON.name || metadata.name}">
      <div class="card-body">
        <p class="gradient-text"><strong>Nazwa:</strong> ${metadataJSON.name || metadata.name}</p>
        <p class="gradient-text"><strong>Symbol:</strong> ${metadataJSON.symbol || metadata.symbol}</p>
    
        <p style="color: #f1c40f; font-size: 1rem; font-weight: bold;"><strong>Section:</strong> ${sectionValue}</p>
        <p style="color: #f1c40f; font-size: 1rem; font-weight: bold;"><strong>Row:</strong> ${rowValue}</p>
        <p style="color: #f1c40f; font-size: 1rem; font-weight: bold;"><strong>Seat:</strong> ${seatValue}</p>
        <p style="color: #f1c40f; font-size: 1rem; font-weight: bold;"><strong>Date of the event:</strong> ${dateValue}</p>
    
        <p class="gradient-text"><strong>Opis:</strong> ${metadataJSON.description || "Brak opisu"}</p>
        <p class="gradient-text"><strong>URI:</strong> <a href="${fixIpfsUri(metadata.uri)}" target="_blank" style="color:#1e90ff">${fixIpfsUri(metadata.uri).slice(0, 35)}...</a></p>
    
        <button class="btn btn-invilink mt-3" onclick='activateNFT("${eventIdFromName}", "${sectionValue}", ${rowValue}, ${seatValue})'>Aktywuj NFT</button>
      </div>
    `;
    
    
    
    nftContainer.appendChild(nftDiv);
  }
}

// Funkcja obliczająca PDA dla konta TicketStatus
async function getTicketStatusPDA(eventId, section, row, seat) {

  const constants = await getConstants();
  const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
  await initConnection();

  const encoder = new TextEncoder();
  const seed1 = encoder.encode("ticket_status");
  const seed2 = encoder.encode(eventId);
  const seed3 = encoder.encode(section);
  const seed4 = new Uint8Array([row]);
  const seed5 = new Uint8Array([seat]);
  const [ticketStatusPDA] = await solanaWeb3.PublicKey.findProgramAddress(
    [seed1, seed2, seed3, seed4, seed5],
    PROGRAM_ID
  );
  return ticketStatusPDA;
}

// Funkcja aktywująca NFT poprzez wywołanie kontraktu activate_ticket
async function activateNFT(eventId, section, row, seat) {

  const constants = await getConstants();
  const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
  const NETWORK = constants.NETWORK;
  const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  await initConnection();

  try {
    await window.phantom.solana.connect();
    walletPublicKey = window.phantom.solana.publicKey;
    const ticketStatusPDA = await getTicketStatusPDA(eventId, section, row, seat);
    console.log("TicketStatus PDA do aktywacji: " + ticketStatusPDA.toBase58());

    const data = buildActivateTicketData(eventId, section, row, seat);

    const instruction = new solanaWeb3.TransactionInstruction({
      keys: [
        { pubkey: ticketStatusPDA, isWritable: true, isSigner: false },
        { pubkey: walletPublicKey, isWritable: false, isSigner: true }
      ],
      programId: PROGRAM_ID,
      data: data,
    });

    const transaction = new solanaWeb3.Transaction().add(instruction);
    transaction.feePayer = walletPublicKey;
    const { blockhash } = await connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;

    const signedTx = await window.phantom.solana.signTransaction(transaction);
    const txSignature = await connection.sendRawTransaction(signedTx.serialize());
    await connection.confirmTransaction(txSignature, "confirmed");
    console.log("NFT aktywowane! Tx Sig: " + txSignature);
    alert("NFT aktywowane! Tx Sig: " + txSignature);
  } catch (err) {
    console.log("Błąd aktywacji NFT: " + err.message);
    alert("Błąd aktywacji NFT: " + err.message);
  }
}