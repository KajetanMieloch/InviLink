async function loadEvents() {
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

    await initConnection();

    const [registryPDA] = await solanaWeb3.PublicKey.findProgramAddress(
    [new TextEncoder().encode("event_registry")],
    PROGRAM_ID
    );
    console.log("Registry PDA: " + registryPDA.toBase58());

    const regAccount = await connection.getAccountInfo(registryPDA);
    if (!regAccount) { alert("Nie znaleziono konta rejestru eventów."); return; }
    const registry = decodeRegistry(regAccount.data);
    console.log("Liczba zapisanych eventów: " + registry.eventCount);

    const tbody = document.querySelector("#eventsTable tbody");
    tbody.innerHTML = "";

    for (let pubkeyStr of registry.events) {
    const eventPubkey = new solanaWeb3.PublicKey(pubkeyStr);
    const eventAcc = await connection.getAccountInfo(eventPubkey);
    if (eventAcc) {
        const eventData = decodeEvent(eventAcc.data);
        const eventDateStr = new Date(eventData.event_date * 1000).toLocaleDateString();

        const tr = document.createElement("tr");
        tr.innerHTML = `
        <td>${eventData.event_id}</td>
        <td>${eventData.name}</td>
        <td>${eventDateStr}</td>
        <td>${eventData.ticket_price}</td>
        <td>${eventData.available_tickets}</td>
        <td>${eventData.sold_tickets}</td>
        <td>${eventData.active}</td>
        <td>${eventData.organizer}</td>
        <td id="action-${pubkeyStr}"></td>
        `;
        tbody.appendChild(tr);
        if (eventData.organizer === walletPublicKey.toBase58()) {
        const actionTd = document.getElementById("action-" + pubkeyStr);
            const btnDeactivate = document.createElement("button");
            btnDeactivate.textContent = "Dezaktywuj";
            btnDeactivate.onclick = () => deactivateEvent(eventPubkey);
            actionTd.appendChild(btnDeactivate);
        }
    }
    }
}


