async function loadEvent(eventId) {
    
    // Obliczamy PDA dla eventu: seeds = [ "event", eventId ]
    const seed1 = new TextEncoder().encode("event");
    const seed2 = new TextEncoder().encode(eventId);
    const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
    [seed1, seed2],
    PROGRAM_ID
    );
    console.log("Obliczone Event PDA: " + eventPDA.toBase58());

    const eventAcc = await connection.getAccountInfo(eventPDA);
    if (!eventAcc) {
    console.log("Nie znaleziono konta eventu przy PDA: " + eventPDA.toBase58());
    document.getElementById("eventDetails").textContent = "Nie znaleziono eventu o podanym Event ID.";
    document.getElementById("seatingCounter").innerHTML = "";
    document.getElementById("sectionsTable").innerHTML = "";
    document.getElementById("addSectionForm").style.display = "none";
    return;
    }
    console.log("Znaleziono konto eventu.");
    const eventData = decodeEvent(eventAcc.data);
    console.log("Zdekodowane dane eventu: " + JSON.stringify(eventData, null, 2));
    showEventInfo(eventData);
    window.currentEvent = eventData;

    if (eventData.seating_type === 1 || eventData.seating_type === 2) {
    document.getElementById("addSectionForm").style.display = "block";
    await loadSeatingSections(eventId);
    } else {
    document.getElementById("seatingCounter").innerHTML = "";
    document.getElementById("sectionsTable").innerHTML = "<p>Event jest typu open-space (brak seating mapy).</p>";
    document.getElementById("addSectionForm").style.display = "none";
    }
}

function showEventInfo(eventData) {
    // Konwersja daty z UNIX timestamp na czytelny format (np. dd.mm.yyyy)
    const eventDateStr = new Date(eventData.event_date * 1000).toLocaleDateString();
    const ed = document.getElementById("eventDetails");
    ed.innerHTML = `
      <h2>Event: ${eventData.name}</h2>
      <p><b>EventID:</b> ${eventData.event_id}</p>
      <p><b>Organizer:</b> ${eventData.organizer}</p>
      <p><b>Data:</b> ${eventDateStr}</p>
      <p><b>Ticket Price:</b> ${eventData.ticket_price} lamportów</p>
      <p><b>Available Tickets:</b> ${eventData.available_tickets}</p>
      <p><b>Sold Tickets:</b> ${eventData.sold_tickets}</p>
      <p><b>Seating Type:</b> ${eventData.seating_type}</p>
      <p><b>Active:</b> ${eventData.active}</p>
    `;
}