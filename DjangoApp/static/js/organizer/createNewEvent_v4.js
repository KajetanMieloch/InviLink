async function createNewEvent() {

    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const CREATE_EVENT_SEATING_DISCRIMINATOR = new Uint8Array([235, 92, 108, 158, 159, 112, 128, 66]);
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

    try {
      await initConnection();
      const eventName = document.getElementById("eventName").value.trim();
      const eventDateInput = document.getElementById("eventDate").value;
      const ticketPriceSol = parseFloat(document.getElementById("ticketPrice").value);
      const availableTickets = parseInt(document.getElementById("availableTickets").value);
  
      if (!eventName || !eventDateInput || isNaN(ticketPriceSol) || isNaN(availableTickets)) {
        alert("All fields must be filled!");
        return;
      }
  
      // Convert date to UNIX timestamp (seconds)
      const eventDateTimestamp = Math.floor(new Date(eventDateInput).getTime() / 1000);
      console.log("Event date (timestamp): " + eventDateTimestamp);
  
      // Convert SOL to lamports
      const ticketPriceLamports = ticketPriceSol * solanaWeb3.LAMPORTS_PER_SOL;
      const ticketPriceBN = new BN(ticketPriceLamports.toString());
      const availableTicketsBN = new BN(availableTickets.toString());
  
      // Generate event_id
      const eventIdGenerated = await generateEventId(eventName, eventDateTimestamp, walletPublicKey);
      console.log("Generated event_id: " + eventIdGenerated);
  
      // Calculate PDA for event and seating_map
      let eventPDA, seatingMapPDA, organizersPoolPDA, registryPDA;

        [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
            [new TextEncoder().encode("event"), new TextEncoder().encode(eventIdGenerated)],
            PROGRAM_ID
        );
        [seatingMapPDA] = await solanaWeb3.PublicKey.findProgramAddress(
            [new TextEncoder().encode("seating_map"), new TextEncoder().encode(eventIdGenerated)],
            PROGRAM_ID
        );
        [organizersPoolPDA] = await solanaWeb3.PublicKey.findProgramAddress(
            [new TextEncoder().encode("organizers_pool")],
            PROGRAM_ID
        );
        [registryPDA] = await solanaWeb3.PublicKey.findProgramAddress(
            [new TextEncoder().encode("event_registry")],
            PROGRAM_ID
        );
  
      // Build instruction data (discriminator + serialized args)
      const discriminator = CREATE_EVENT_SEATING_DISCRIMINATOR;
      const serializedArgs = serializeCreateEventSeatingArgs({
        event_id: eventIdGenerated,
        name: eventName,
        event_date: eventDateTimestamp,
        ticket_price: ticketPriceBN,
        available_tickets: availableTicketsBN
      });
      const instructionData = new Uint8Array(discriminator.length + serializedArgs.length);
      instructionData.set(discriminator, 0);
      instructionData.set(serializedArgs, discriminator.length);
  
      // Prepare list of accounts:
      // 0. event (PDA, writable)
      // 1. seating_map (PDA, writable)
      // 2. organizers_pool (PDA, writable)
      // 3. registry (PDA, writable)
      // 4. organizer (wallet, signer, writable)
      // 5. system_program
      const keys = [
        { pubkey: eventPDA, isSigner: false, isWritable: true },
        { pubkey: seatingMapPDA, isSigner: false, isWritable: true },
        { pubkey: organizersPoolPDA, isSigner: false, isWritable: true },
        { pubkey: registryPDA, isSigner: false, isWritable: true },
        { pubkey: walletPublicKey, isSigner: true, isWritable: true },
        { pubkey: solanaWeb3.SystemProgram.programId, isSigner: false, isWritable: false }
      ];
  
      // Create and send transaction
      const transactionInstruction = new solanaWeb3.TransactionInstruction({
        keys,
        programId: PROGRAM_ID,
        data: instructionData
      });
  
      let transaction = new solanaWeb3.Transaction().add(transactionInstruction);
      transaction.feePayer = walletPublicKey;
      const { blockhash } = await connection.getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
  
      const signedTransaction = await provider.signTransaction(transaction);
      const txSignature = await connection.sendRawTransaction(signedTransaction.serialize());
      console.log("Transaction sent, signature: " + txSignature);
      await connection.confirmTransaction(txSignature, "confirmed");
      console.log("Event created! Tx Sig: " + txSignature);
      alert("Event created! Tx Sig: " + txSignature);
    } catch (err) {
      console.log("Error: " + err.message);
      alert("Error: " + err.message);
    }
  }