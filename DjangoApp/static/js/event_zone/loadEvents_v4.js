async function loadEvents() {
  const constants = await getConstants();
  const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
  const NETWORK = constants.NETWORK;
  const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

  await initConnection();

  const [registryPDA] = await solanaWeb3.PublicKey.findProgramAddress([
    new TextEncoder().encode("event_registry")
  ], PROGRAM_ID);

  const regAccount = await connection.getAccountInfo(registryPDA);
  if (!regAccount) {
    showErrorAlertwithMSG("No events found.");
    return;
  }

  const registry = decodeRegistry(regAccount.data);
  const container = document.getElementById("eventCards");

  for (let pubkeyStr of registry.events) {
    const eventPubkey = new solanaWeb3.PublicKey(pubkeyStr);
    const eventAcc = await connection.getAccountInfo(eventPubkey);

    if (eventAcc) {
      const eventData = decodeEvent(eventAcc.data);
      if (!eventData.active) continue;

      const dateStr = new Date(eventData.event_date * 1000).toLocaleDateString();

      const col = document.createElement("div");
      col.className = "col-md-4 mb-4";

      col.innerHTML = `
        <div class="card">
          <img src="/static/images/favicon.ico" class="card-img-top" alt="Event Image">
          <div class="card-body">
            <h5 class="card-title gradient-text">${eventData.name}</h5>
            <p class="gradient-text small mb-1"><strong>${dateStr}</strong></p>
            <p class="gradient-text">Tickets: ${eventData.available_tickets}</p>
            <p class="gradient-text text-muted small">Organizer: ${eventData.organizer}</p>
            <button class="btn btn-invilink mt-3" onclick="window.location.href='/explore/event/${eventData.event_id}'">Pokaż szczegóły</button>
          </div>
        </div>
      `;

      container.appendChild(col);
    }
  }
}
