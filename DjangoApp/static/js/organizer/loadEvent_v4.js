async function loadEvent(eventId) {
  const constants = await getConstants();
  const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
  const NETWORK = constants.NETWORK;
  const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  window.currentEvent = null;

  await initConnection();

  // Compute PDA for the event: seeds = [ "event", eventId ]
  const seed1 = new TextEncoder().encode("event");
  const seed2 = new TextEncoder().encode(eventId);
  const [eventPDA] = await solanaWeb3.PublicKey.findProgramAddress(
    [seed1, seed2],
    PROGRAM_ID
  );
  console.log("Computed Event PDA: " + eventPDA.toBase58());

  const eventAcc = await connection.getAccountInfo(eventPDA);
  if (!eventAcc) {
    console.log("Event account not found at PDA: " + eventPDA.toBase58());
    document.getElementById("eventDetails").textContent = "Event with the given Event ID was not found.";
    document.getElementById("seatingCounter").innerHTML = "";
    document.getElementById("sectionsTable").innerHTML = "";
    document.getElementById("addSectionForm").style.display = "none";
    return;
  }

  console.log("Event account found.");
  const eventData = decodeEvent(eventAcc.data);
  console.log("Decoded event data: " + JSON.stringify(eventData, null, 2));
  showEventInfo(eventData);
  window.currentEvent = eventData;

  if (eventData.seating_type === 1 || eventData.seating_type === 2) {
    document.getElementById("addSectionForm").style.display = "block";
    await loadSeatingSections(eventId);
  } else {
    document.getElementById("seatingCounter").innerHTML = "";
    document.getElementById("sectionsTable").innerHTML = "<p>This event is open-space type (no seating map).</p>";
    document.getElementById("addSectionForm").style.display = "none";
  }
}

function showEventInfo(eventData) {
  // Convert UNIX timestamp to readable format (e.g. dd.mm.yyyy)
  const eventDateStr = new Date(eventData.event_date * 1000).toLocaleDateString();
  const ed = document.getElementById("eventDetails");
  ed.innerHTML = `
    <h2>Event: ${eventData.name}</h2>
    <p><b>Event ID:</b> ${eventData.event_id}</p>
    <p><b>Organizer:</b> ${eventData.organizer}</p>
    <p><b>Date:</b> ${eventDateStr}</p>
    <p><b>Available Tickets:</b> ${eventData.available_tickets}</p>
    <p><b>Sold Tickets:</b> ${eventData.sold_tickets}</p>
    <p><b>Seating Type:</b> ${eventData.seating_type}</p>
    <p><b>Active:</b> ${eventData.active}</p>
  `;
}