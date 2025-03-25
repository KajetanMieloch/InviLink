async function loadEvents() {
    const constants = await getConstants();
    const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
    const NETWORK = constants.NETWORK;
    const connection = new solanaWeb3.Connection(NETWORK, "confirmed");
  
    await initConnection();
  
    // Get the PDA for the event registry
    const [registryPDA] = await solanaWeb3.PublicKey.findProgramAddress(
      [new TextEncoder().encode("event_registry")],
      PROGRAM_ID
    );
    console.log("Registry PDA: " + registryPDA.toBase58());
  
    const regAccount = await connection.getAccountInfo(registryPDA);
    if (!regAccount) {
      alert("Event registry account not found.");
      return;
    }
  
    const registry = decodeRegistry(regAccount.data);
    console.log("Total registered events: " + registry.eventCount);
  
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
          <td>${eventData.available_tickets}</td>
          <td>${eventData.sold_tickets}</td>
          <td>${eventData.active}</td>
          <td>${eventData.organizer}</td>
          <td id="action-${pubkeyStr}"></td>
        `;
  
        // Show only events managed by the current wallet
        if (eventData.organizer === walletPublicKey.toBase58()) {
          tbody.appendChild(tr);
          const actionTd = document.getElementById("action-" + pubkeyStr);
          const btnManage = document.createElement("button");
          btnManage.textContent = "Manage Event";
          btnManage.onclick = () => {
            window.location.href = "manage/" + eventData.event_id;
          };
          actionTd.appendChild(btnManage);
        }
      }
    }
  }  