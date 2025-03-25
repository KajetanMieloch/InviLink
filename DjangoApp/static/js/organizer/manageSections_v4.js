async function addSeatingSection() {
  
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");  
  
    const sectionName = document.getElementById("sectionNameInput").value.trim();
    const rows = parseInt(document.getElementById("rowsInput").value);
    const seatsPerRow = parseInt(document.getElementById("seatsPerRowInput").value);
    const sectionType = parseInt(document.getElementById("sectionTypeInput").value);
    const ticketPrice = parseInt(document.getElementById("ticketPriceInput").value);
  
    if (!sectionName || isNaN(rows) || isNaN(seatsPerRow) || isNaN(sectionType) || isNaN(ticketPrice)) {
      alert("Wszystkie pola muszą być wypełnione!");
      return;
    }
    
    if (!window.currentEvent) {
      alert("Najpierw wczytaj event!");
      return;
    }
    
    const eventId = window.currentEvent.event_id;
    console.log("Dodaję sekcję: " + sectionName + " dla eventu: " + eventId);
    
    const seed1 = new TextEncoder().encode("seating_map");
    const seed2 = new TextEncoder().encode(eventId);
    const [seatingMapPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [seed1, seed2],
      PROGRAM_ID
    );
    console.log("Obliczone Seating Map PDA: " + seatingMapPDA.toBase58());
    
    const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("event"), new TextEncoder().encode(eventId)],
      PROGRAM_ID
    );
    
    const [seatingSectionPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [
        new TextEncoder().encode("seating_section"),
        eventPDA.toBytes(),
        new TextEncoder().encode(sectionName)
      ],
      PROGRAM_ID
    );
    console.log("Obliczone Seating Section PDA: " + seatingSectionPDA.toBase58());
    
    const serializedArgs = serializeInitializeSeatingSectionArgs({
        section_name: sectionName,
        section_type: sectionType,
        rows: rows,
        seats_per_row: seatsPerRow,
        ticket_price: ticketPrice
    });
    
    const INITIALIZE_SEATING_SECTION_DISCRIMINATOR = new Uint8Array([151,223,44,246,213,70,7,65]);
    const instructionData = new Uint8Array(INITIALIZE_SEATING_SECTION_DISCRIMINATOR.length + serializedArgs.length);
    instructionData.set(INITIALIZE_SEATING_SECTION_DISCRIMINATOR, 0);
    instructionData.set(serializedArgs, INITIALIZE_SEATING_SECTION_DISCRIMINATOR.length);
    
    const keys = [
      { pubkey: seatingMapPDA, isSigner: false, isWritable: true },
      { pubkey: seatingSectionPDA, isSigner: false, isWritable: true },
      { pubkey: eventPDA, isSigner: false, isWritable: false },
      { pubkey: walletPublicKey, isSigner: true, isWritable: true },
      { pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false }
    ];
    
    const txInstruction = new solanaWeb3.TransactionInstruction({
      keys,
      programId: PROGRAM_ID,
      data: instructionData
    });
    
    try {
      let transaction = new solanaWeb3.Transaction().add(txInstruction);
      transaction.feePayer = walletPublicKey;
      const { blockhash } = await connection.getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
      const signedTransaction = await provider.signTransaction(transaction);
      const txSignature = await connection.sendRawTransaction(signedTransaction.serialize());
      console.log("Transakcja dodania sekcji wysłana, signature: " + txSignature);
      await connection.confirmTransaction(txSignature, "confirmed");
      console.log("Sekcja została dodana! Tx Sig: " + txSignature);
      alert("Sekcja została dodana! Tx Sig: " + txSignature);
      await loadSeatingSections(eventId);
    } catch (err) {
      console.log("Błąd dodawania sekcji: " + err.message);
      alert("Błąd dodawania sekcji: " + err.message);
    }
  }

async function editSection(sectionName) {

    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

    if (!window.currentEvent) {
      alert("Najpierw wczytaj event!");
      return;
    }
    const eventId = window.currentEvent.event_id;
    let newRowsInput = prompt("Podaj nową liczbę wierszy (pozostaw puste, aby nie zmieniać):");
    let newSeatsInput = prompt("Podaj nową liczbę miejsc w wierszu (pozostaw puste, aby nie zmieniać):");
    let newTypeInput = prompt("Podaj nowy typ sekcji (1 = Numerowane, 0 = Stojące) (pozostaw puste, aby nie zmieniać):");
    let newTicketPriceInput = prompt("Podaj nową cenę biletu (pozostaw puste, aby nie zmieniać):");
    
    const newRows = newRowsInput ? parseInt(newRowsInput) : null;
    const newSeats = newSeatsInput ? parseInt(newSeatsInput) : null;
    const newType = newTypeInput ? parseInt(newTypeInput) : null;
    const newTicketPrice = newTicketPriceInput ? parseInt(newTicketPriceInput) : null;
    
    const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("event"), new TextEncoder().encode(eventId)],
      PROGRAM_ID
    );
    
    const [seatingMapPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("seating_map"), new TextEncoder().encode(eventId)],
      PROGRAM_ID
    );
    
    const [seatingSectionPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("seating_section"), eventPDA.toBytes(), new TextEncoder().encode(sectionName)],
      PROGRAM_ID
    );
    console.log("Obliczone Seating Section PDA (update): " + seatingSectionPDA.toBase58());
    
    const newRowsBytes = serializeOptionU8(newRows);
    const newSeatsBytes = serializeOptionU8(newSeats);
    const newTypeBytes = serializeOptionU8(newType);
    const newTicketPriceBytes = serializeOptionU64(newTicketPrice);
    
    const totalLen = newRowsBytes.length + newSeatsBytes.length + newTypeBytes.length + newTicketPriceBytes.length;
    const argsBuffer = new Uint8Array(totalLen);
    let offset = 0;
    argsBuffer.set(newRowsBytes, offset); offset += newRowsBytes.length;
    argsBuffer.set(newSeatsBytes, offset); offset += newSeatsBytes.length;
    argsBuffer.set(newTypeBytes, offset); offset += newTypeBytes.length;
    argsBuffer.set(newTicketPriceBytes, offset);
    
    const UPDATE_SEATING_SECTION_DISCRIMINATOR = new Uint8Array([46,155,128,9,243,228,210,182]);
    const instructionData = new Uint8Array(UPDATE_SEATING_SECTION_DISCRIMINATOR.length + argsBuffer.length);
    instructionData.set(UPDATE_SEATING_SECTION_DISCRIMINATOR, 0);
    instructionData.set(argsBuffer, UPDATE_SEATING_SECTION_DISCRIMINATOR.length);
    
    const keys = [
      { pubkey: seatingMapPDA, isSigner: false, isWritable: true },
      { pubkey: seatingSectionPDA, isSigner: false, isWritable: true },
      { pubkey: eventPDA, isSigner: false, isWritable: false },
      { pubkey: walletPublicKey, isSigner: true, isWritable: true },
      { pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false }
    ];
    
    const txInstruction = new solanaWeb3.TransactionInstruction({
      keys,
      programId: PROGRAM_ID,
      data: instructionData
    });
    
    try {
      let transaction = new solanaWeb3.Transaction().add(txInstruction);
      transaction.feePayer = walletPublicKey;
      const { blockhash } = await connection.getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
      const signedTransaction = await provider.signTransaction(transaction);
      const txSignature = await connection.sendRawTransaction(signedTransaction.serialize());
      console.log("Transakcja update sekcji wysłana, signature: " + txSignature);
      await connection.confirmTransaction(txSignature, "confirmed");
      console.log("Sekcja została zaktualizowana! Tx Sig: " + txSignature);
      alert("Sekcja została zaktualizowana! Tx Sig: " + txSignature);
      await loadSeatingSections(eventId);
    } catch (err) {
      console.log("Błąd update sekcji: " + err.message);
      alert("Błąd update sekcji: " + err.message);
    }
  }
  
  async function deleteSection(sectionName) {

    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

    if (!window.currentEvent) {
      alert("Najpierw wczytaj event!");
      return;
    }
    const eventId = window.currentEvent.event_id;
    console.log("Usuwam sekcję: " + sectionName + " dla eventu: " + eventId);
    
    const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("event"), new TextEncoder().encode(eventId)],
      PROGRAM_ID
    );
    
    const [seatingMapPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("seating_map"), new TextEncoder().encode(eventId)],
      PROGRAM_ID
    );
    
    const [seatingSectionPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("seating_section"), eventPDA.toBytes(), new TextEncoder().encode(sectionName)],
      PROGRAM_ID
    );
    console.log("Obliczone Seating Section PDA (delete): " + seatingSectionPDA.toBase58());
    
    const REMOVE_SEATING_SECTION_DISCRIMINATOR = new Uint8Array([26,199,35,22,4,211,10,86]);
    const instructionData = REMOVE_SEATING_SECTION_DISCRIMINATOR;
    
    const keys = [
      { pubkey: seatingMapPDA, isSigner: false, isWritable: true },
      { pubkey: seatingSectionPDA, isSigner: false, isWritable: true },
      { pubkey: eventPDA, isSigner: false, isWritable: false },
      { pubkey: walletPublicKey, isSigner: true, isWritable: true },
      { pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false }
    ];
    
    const txInstruction = new solanaWeb3.TransactionInstruction({
      keys,
      programId: PROGRAM_ID,
      data: instructionData
    });
    
    try {
      let transaction = new solanaWeb3.Transaction().add(txInstruction);
      transaction.feePayer = walletPublicKey;
      const { blockhash } = await connection.getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
      const signedTransaction = await provider.signTransaction(transaction);
      const txSignature = await connection.sendRawTransaction(signedTransaction.serialize());
      console.log("Transakcja usunięcia sekcji wysłana, signature: " + txSignature);
      await connection.confirmTransaction(txSignature, "confirmed");
      console.log("Sekcja została usunięta! Tx Sig: " + txSignature);
      alert("Sekcja została usunięta! Tx Sig: " + txSignature);
      await loadSeatingSections(eventId);
    } catch (err) {
      console.log("Błąd usuwania sekcji: " + err.message);
      alert("Błąd usuwania sekcji: " + err.message);
    }
  }