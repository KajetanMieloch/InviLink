    // Stałe – adresy programu i inne
    const MINT_TICKET_DISCRIMINATOR = new Uint8Array([212, 78, 142, 4, 188, 28, 203, 17]);
    const METADATA_PROGRAM_ID = new solanaWeb3.PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
    const TOKEN_PROGRAM_ID = new solanaWeb3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    const ASSOCIATED_TOKEN_PROGRAM_ID = new solanaWeb3.PublicKey("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
    // Stałe MASTER_ACCOUNT zgodne z Twoim kontraktem
    const MASTER_ACCOUNT = new solanaWeb3.PublicKey("4Wg5ZqjS3AktHzq34hK1T55aFNKSjBpmJ3PyRChpPNDh");

    let provider, walletPublicKey;
    let currentEventID = null; // ustawiane przy wpisaniu EventID
    let eventData = null;      // dane eventu pobrane z łańcucha
  
    


    // Oblicza PDA dla mint NFT – seed'y: ["mint_ticket", event_id, event_name, section_name, [row], [seat]]
    async function getTestMintPDA(event_id, event_name, section_name, row, seat) {
      const encoder = new TextEncoder();
      const seed1 = encoder.encode("mint_ticket");
      const seed2 = encoder.encode(event_id);
      const seed3 = encoder.encode(event_name);
      const seed4 = encoder.encode(section_name);
      const seed5 = new Uint8Array([row]);
      const seed6 = new Uint8Array([seat]);
      const [mintPDA, bump] = await solanaWeb3.PublicKey.findProgramAddress(
        [seed1, seed2, seed3, seed4, seed5, seed6],
        PROGRAM_ID
      );
      console.log("Obliczony mint PDA: " + mintPDA.toBase58() + " (bump: " + bump + ")");
      return { mintPDA, bump };
    }
    async function getAssociatedTokenAddress(owner, mint) {
      return (await solanaWeb3.PublicKey.findProgramAddress(
        [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
        ASSOCIATED_TOKEN_PROGRAM_ID
      ))[0];
    }
    async function getMetadataPDA(mint) {
      const seed1 = new TextEncoder().encode("metadata");
      const seed2 = METADATA_PROGRAM_ID.toBytes();
      const seed3 = mint.toBytes();
      const [metadataPDA] = await solanaWeb3.PublicKey.findProgramAddress(
        [seed1, seed2, seed3],
        METADATA_PROGRAM_ID
      );
      return metadataPDA;
    }
    // Funkcja do obliczenia PDA dla konta seating_section
    async function getSeatingSectionPDA(eventPDA, sectionName) {
      const encoder = new TextEncoder();
      const seed = encoder.encode(sectionName);
      const [seatingSectionPDA, bump] = await solanaWeb3.PublicKey.findProgramAddress(
        [new TextEncoder().encode("seating_section"), eventPDA.toBytes(), seed],
        PROGRAM_ID
      );
      console.log("Obliczony Seating Section PDA: " + seatingSectionPDA.toBase58() + " (bump: " + bump + ")");
      return seatingSectionPDA;
    }

    // Ładowanie eventu – pobieramy event oraz seating_map
    async function loadEvent(currentEventID) {
      const constants = await getConstants();
      const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
      const NETWORK = constants.NETWORK;
      const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
      await initConnection();

        console.log("Ładowanie eventu: " + currentEventID);
        const eventSeed1 = new TextEncoder().encode("event");
        const eventSeed2 = new TextEncoder().encode(currentEventID);
        const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress([eventSeed1, eventSeed2], PROGRAM_ID);
        console.log("Event PDA: " + eventPDA.toBase58());
        const eventAcc = await connection.getAccountInfo(eventPDA);
        if (!eventAcc) {
          console.log("Nie znaleziono eventu.");
          document.getElementById("eventDetails").innerText = "Nie znaleziono eventu.";
          return;
        }
        eventData = decodeEventNFT(eventAcc.data);
        console.log("Dane eventu: " + JSON.stringify(eventData, null, 2));
        displayEvent(eventData);
        const seatingSeed1 = new TextEncoder().encode("seating_map");
        const seatingSeed2 = new TextEncoder().encode(currentEventID);
        const [seatingMapPDA] = await solanaWeb3.PublicKey.findProgramAddress([seatingSeed1, seatingSeed2], PROGRAM_ID);
        console.log("Seating Map PDA: " + seatingMapPDA.toBase58());
        const seatingMapAcc = await connection.getAccountInfo(seatingMapPDA);
        if (!seatingMapAcc) {
          console.log("Nie znaleziono mapy miejsc.");
          return;
        }
        eventData.seating_map = decodeSeatingMap(seatingMapAcc.data);
        console.log("SeatingMap: " + JSON.stringify(eventData.seating_map, null, 2));
        loadSections(eventData.seating_map.sections);
      } 

    // Funkcja dekodująca event – rozszerzona o event_date


    function displayEvent(ev) {
      // Konwersja event_date na czytelny format
      const eventDateStr = new Date(ev.event_date * 1000).toLocaleDateString();
      const detailsDiv = document.getElementById("eventDetails");
      detailsDiv.innerHTML = `
        <h2>${ev.name}</h2>
        <p><strong>Event ID:</strong> ${ev.event_id}</p>
        <p><strong>Organizator:</strong> ${ev.organizer}</p>
        <p><strong>Data:</strong> ${eventDateStr}</p>
        <p><strong>Cena biletu:</strong> ${ev.ticket_price} lamportów</p>
        <p><strong>Dostępne bilety:</strong> ${ev.available_tickets}</p>
        <p><strong>Sprzedane bilety:</strong> ${ev.sold_tickets}</p>
        <p><strong>Rodzaj miejsc:</strong> ${ev.seating_type === 1 ? "Numerowane" : "Stojące"}</p>
        <p><strong>Aktywny:</strong> ${ev.active}</p>
      `;
    }

    async function loadSections(sectionPubkeys) {

      const constants = await getConstants();
      const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
      const NETWORK = constants.NETWORK;
      const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
      await initConnection();

        const container = document.getElementById("sectionsContainer");
        container.innerHTML = "";
        if (sectionPubkeys.length === 0) {
          container.innerHTML = "<p>Brak sekcji.</p>";
          return;
        }
        for (let pubkeyStr of sectionPubkeys) {
          const sectionAcc = await connection.getAccountInfo(new solanaWeb3.PublicKey(pubkeyStr));
          if (sectionAcc) {
            const sectionData = decodeSeatingSectionAccount(sectionAcc.data);
            let div = document.createElement("div");
            div.innerHTML = `<h3>Sekcja: ${sectionData.section_name}</h3>`;
            if (sectionData.section_type === 1) {
              createInteractiveMapForElement(div, sectionData);
            } else {
              const totalSeats = sectionData.rows * sectionData.seats_per_row;
              const freeSeats = sectionData.seat_status.filter(status => status === 0).length;
              div.innerHTML += `<div class="standing-section">
                  <span class="standing-info">Wolne miejsca: ${freeSeats} / ${totalSeats}</span>
                  <button onclick='buyStandingTicket("${sectionData.section_name}")'>Kup bilet</button>
                </div>`;
            }
            container.appendChild(div);
          }
        }
      }

    function createInteractiveMapForElement(container, sectionData) {
      const grid = document.createElement("div");
      grid.className = "seat-grid";
      grid.style.gridTemplateColumns = `repeat(${sectionData.seats_per_row}, 30px)`;
      const totalSeats = sectionData.rows * sectionData.seats_per_row;
      for (let i = 0; i < totalSeats; i++) {
        const btn = document.createElement("button");
        btn.className = "seat-button";
        if (sectionData.seat_status[i] === 0) {
          btn.classList.add("seat-available");
          btn.innerText = i;
          btn.onclick = () => mintNFTForSeat(sectionData, i);
        } else {
          btn.classList.add("seat-taken");
          btn.disabled = true;
          btn.innerText = "X";
        }
        grid.appendChild(btn);
      }
      container.appendChild(grid);
    }

    function mintNFTForSeat(sectionData, seatIndex) {
      const row = Math.floor(seatIndex / sectionData.seats_per_row);
      const seat = seatIndex % sectionData.seats_per_row;
      console.log(`Mint NFT dla sekcji "${sectionData.section_name}", row ${row}, seat ${seat}`);
      processMintTicketNFT(sectionData.section_name, row, seat);
    }

    async function buyStandingTicket(sectionName) {
      try {
        const eventPDA = await getEventPDA(eventData.event_id);
        const seatingSectionPDA = await getSeatingSectionPDA(eventPDA, sectionName);
        const sectionAcc = await connection.getAccountInfo(seatingSectionPDA);
        if (!sectionAcc) throw new Error("Nie znaleziono danych sekcji.");
        const sectionData = decodeSeatingSectionAccount(sectionAcc.data);
        const freeIndices = [];
        sectionData.seat_status.forEach((status, index) => { if (status === 0) freeIndices.push(index); });
        if (freeIndices.length === 0) { alert("Brak wolnych miejsc w sekcji " + sectionName); return; }
        const randomIndex = freeIndices[Math.floor(Math.random() * freeIndices.length)];
        const { row, seat } = validateSeatCoordinates(sectionData, randomIndex);
        console.log(`Kupuję bilet dla sekcji stojącej "${sectionName}" – row: ${row}, seat: ${seat}`);
        processMintTicketNFT(sectionName, row, seat);
      } catch (err) {
        console.log("Błąd kupowania biletu: " + err.message);
        alert("Błąd kupowania biletu: " + err.message);
      }
    }

    function validateSeatCoordinates(sectionData, seatIndex) {
      const maxSeats = sectionData.rows * sectionData.seats_per_row;
      if (seatIndex < 0 || seatIndex >= maxSeats) throw new Error(`Nieprawidłowy indeks miejsca: ${seatIndex}`);
      return { row: Math.floor(seatIndex / sectionData.seats_per_row), seat: seatIndex % sectionData.seats_per_row };
    }

    async function getEventPDA(eventId) {
      const seed1 = new TextEncoder().encode("event");
      const seed2 = new TextEncoder().encode(eventId);
      const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress([seed1, seed2], PROGRAM_ID);
      return eventPDA;
    }

    // Buduje dane instrukcji – pozostaje bez zmian
    function buildInstructionData(event_id, event_name, section_name, row, seat, ipfs_uri) {
      const eventIdBytes = serializeString(event_id);
      const eventNameBytes = serializeString(event_name);
      const sectionNameBytes = serializeString(section_name);
      const rowByte = serializeU8(row);
      const seatByte = serializeU8(seat);
      const ipfsUriBytes = serializeString(ipfs_uri); // nowy parametr IPFS
      const totalLength = MINT_TICKET_DISCRIMINATOR.length +
                          eventIdBytes.length +
                          eventNameBytes.length +
                          sectionNameBytes.length +
                          rowByte.length +
                          seatByte.length +
                          ipfsUriBytes.length;
      let data = new Uint8Array(totalLength);
      let offset = 0;
      data.set(MINT_TICKET_DISCRIMINATOR, offset);
      offset += MINT_TICKET_DISCRIMINATOR.length;
      data.set(eventIdBytes, offset); offset += eventIdBytes.length;
      data.set(eventNameBytes, offset); offset += eventNameBytes.length;
      data.set(sectionNameBytes, offset); offset += sectionNameBytes.length;
      data.set(rowByte, offset); offset += rowByte.length;
      data.set(seatByte, offset); offset += seatByte.length;
      data.set(ipfsUriBytes, offset);
      return data;
    }

    async function fetchMetadataUri(eventId, section, row, seat, date, name) {
      try {
        const apiEndpoint = "https://invilink.bieda.it/test_blockchain/generate_metadata/";
        const response = await fetch(apiEndpoint, {
          method: "POST",
          headers: {
            "Content-Type": "application/json"
          },
          body: JSON.stringify({
            eventId: eventId,
            section: section,
            row: row,
            seat: seat,
            date: date,
            name: name
          })
        });

        if (!response.ok) {
          throw new Error("Błąd generowania metadanych");
        }

        const data = await response.json();
        return data.uri; // np. "ipfs://Qm..."
      } catch (error) {
        console.log("Błąd pobierania metadata URI: " + error.message);
        throw error;
      }
    }

  // Funkcja pomocnicza do obliczenia PDA dla konta TicketStatus
  async function getTicketStatusPDA(event_id, sectionName, row, seat) {
    const encoder = new TextEncoder();
    const seed1 = encoder.encode("ticket_status");
    const seed2 = encoder.encode(event_id);
    const seed3 = encoder.encode(sectionName);
    const seed4 = new Uint8Array([row]);
    const seed5 = new Uint8Array([seat]);
    const [ticketStatusPDA, bump] = await solanaWeb3.PublicKey.findProgramAddress(
      [seed1, seed2, seed3, seed4, seed5],
      PROGRAM_ID
    );
    console.log("Obliczony TicketStatus PDA: " + ticketStatusPDA.toBase58() + " (bump: " + bump + ")");
    return ticketStatusPDA;
  }

    // Funkcja budująca dane instrukcji dla initialize_ticket_status,
    // która serializuje 4 parametry: event_id, section, row, seat.
    // Ostateczny bufor ma postać:
    // [8 bajtów dyskryminatora] + [serializeString(event_id)] + [serializeString(section)] + [serializeU8(row)] + [serializeU8(seat)]
    function buildInitTicketStatusData(event_id, section, row, seat, eventPubkey) {
    // Dyskryminator (8 bajtów) – musi być zgodny z tym, co ustawiasz w on-chain
    const discriminator = new Uint8Array([228, 37, 235, 14, 223, 66, 40, 21]);
    
    const eventIdBytes = serializeString(event_id); // np. "nGhs9IxZfx5q"
    const sectionBytes = serializeString(section);    // np. "SekcjaA"
    const rowByte = serializeU8(row);                   // 1 bajt
    const seatByte = serializeU8(seat);                 // 1 bajt
    const eventBytes = eventPubkey.toBytes();           // 32 bajty

    // Całkowita długość = 8 + (4 + event_id.length) + (4 + section.length) + 1 + 1 + 32
    const totalLength = discriminator.length + eventIdBytes.length + sectionBytes.length + rowByte.length + seatByte.length + eventBytes.length;
    const buffer = new Uint8Array(totalLength);
    let offset = 0;
    
    buffer.set(discriminator, offset);
    offset += discriminator.length;
    
    buffer.set(eventIdBytes, offset);
    offset += eventIdBytes.length;
    
    buffer.set(sectionBytes, offset);
    offset += sectionBytes.length;
    
    buffer.set(rowByte, offset);
    offset += rowByte.length;
    
    buffer.set(seatByte, offset);
    offset += seatByte.length;
    
    buffer.set(eventBytes, offset);
    
    return buffer;
  }





    // Funkcja realizująca mint NFT
    async function processMintTicketNFT(sectionName, row, seat) {
  try {
    if (!eventData) { 
      console.log("Brak danych eventu!"); 
      return; 
    }
    const event_id = eventData.event_id;
    const event_name = eventData.name;
    console.log(`Mint NFT: event_id=${event_id}, event_name=${event_name}, section=${sectionName}, row=${row}, seat=${seat}`);

    if (!eventData.active) { 
      console.log("Event nie jest aktywny!"); 
      alert("Event nie jest aktywny. Mintowanie niemożliwe."); 
      return; 
    }

    // Pobieramy metadata URI (backend generuje metadane i dodaje je do IPFS)
    const metadataURI = await fetchMetadataUri(event_id, sectionName, row, seat, eventData.event_date, eventData.name);
    console.log("Wygenerowany metadata URI (IPFS): " + metadataURI);

    // Obliczamy potrzebne adresy PDA
    const eventPDA = await getEventPDA(event_id);
    console.log("Obliczone Event PDA: " + eventPDA.toBase58());

    const seatingSeed1 = new TextEncoder().encode("seating_map");
    const seatingSeed2 = new TextEncoder().encode(event_id);
    const [seatingMapPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [seatingSeed1, seatingSeed2],
      PROGRAM_ID
    );
    console.log("Obliczone Seating Map PDA: " + seatingMapPDA.toBase58());
    
    const seatingSectionPDA = await getSeatingSectionPDA(eventPDA, sectionName);
    const { mintPDA } = await getTestMintPDA(event_id, event_name, sectionName, row, seat);
    const tokenAccount = await getAssociatedTokenAddress(walletPublicKey, mintPDA);
    console.log("Obliczony ATA: " + tokenAccount.toBase58());
    const metadataPDA = await getMetadataPDA(mintPDA);
    console.log("Obliczony metadata PDA: " + metadataPDA.toBase58());

    // Budujemy dane instrukcji dla mint NFT
    const ixDataMint = buildInstructionData(event_id, event_name, sectionName, row, seat, metadataURI);

// Obliczamy PDA dla TicketStatus (jak już masz zaimplementowane)
const ticketStatusPDA = await getTicketStatusPDA(event_id, sectionName, row, seat);

// Budujemy dane instrukcji – przekazujemy event_id, sectionName, row, seat oraz eventPDA (jako Pubkey eventu)
const ixDataInit = buildInitTicketStatusData(event_id, sectionName, row, seat, eventPDA);

// Tworzymy instrukcję dla initialize_ticket_status
const initTicketStatusIx = new solanaWeb3.TransactionInstruction({
  keys: [
    { pubkey: ticketStatusPDA, isSigner: false, isWritable: true },
    { pubkey: walletPublicKey, isSigner: true, isWritable: true },
    { pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false }
  ],
  programId: PROGRAM_ID,
  data: ixDataInit,
});



    // Budujemy instrukcję dla mint NFT
    const mintKeys = [
      { pubkey: eventPDA, isSigner: false, isWritable: true },
      { pubkey: walletPublicKey, isSigner: true, isWritable: true },
      { pubkey: seatingMapPDA, isSigner: false, isWritable: true },
      { pubkey: seatingSectionPDA, isSigner: false, isWritable: true },
      { pubkey: mintPDA, isSigner: false, isWritable: true },
      { pubkey: tokenAccount, isSigner: false, isWritable: true },
      { pubkey: metadataPDA, isSigner: false, isWritable: true },
      { pubkey: MASTER_ACCOUNT, isSigner: false, isWritable: true },
      { pubkey: eventData.organizer ? new solanaWeb3.PublicKey(eventData.organizer) : walletPublicKey, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: METADATA_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: solanaWeb3.SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false }
    ];
    
    const mintTicketIx = new solanaWeb3.TransactionInstruction({
      keys: mintKeys,
      programId: PROGRAM_ID,
      data: ixDataMint,
    });

    // Łączymy obie instrukcje w jedną transakcję
    let transaction = new solanaWeb3.Transaction();
    transaction.add(initTicketStatusIx).add(mintTicketIx);
    transaction.feePayer = walletPublicKey;
    const { blockhash } = await connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    
    console.log("Transakcja łączona – podpisuję...");
    const signedTx = await provider.signTransaction(transaction);
    console.log("Transakcja podpisana, wysyłam...");
    const txSignature = await connection.sendRawTransaction(signedTx.serialize());
    console.log("Transakcja wysłana. Signature: " + txSignature);
    await connection.confirmTransaction(txSignature, "confirmed");
    console.log("Transakcja łączona potwierdzona. NFT mintowane!");
    alert("NFT mintowane! Tx Sig: " + txSignature);
    loadEvent();
  } catch (err) {
    console.log("Błąd mintowania NFT: " + err.message);
    alert("Błąd mintowania NFT: " + err.message);
  }
}

