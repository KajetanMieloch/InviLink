async function loadSeatingSections(eventId) {
  const constants = await getConstants();
  const PROGRAM_ID = new solanaWeb3.PublicKey(constants.PROGRAM_ID);
  const NETWORK = constants.NETWORK;
  const connection = new solanaWeb3.Connection(NETWORK, "confirmed");

  const seed1 = new TextEncoder().encode("seating_map");
  const seed2 = new TextEncoder().encode(eventId);
  const [seatingMapPDA] = await solanaWeb3.PublicKey.findProgramAddress(
    [seed1, seed2],
    PROGRAM_ID
  );
  console.log("Computed Seating Map PDA: " + seatingMapPDA.toBase58());

  const seatingMapAcc = await connection.getAccountInfo(seatingMapPDA);
  if (!seatingMapAcc) {
    console.log("SeatingMap account not found for event: " + eventId);
    document.getElementById("seatingCounter").innerHTML = "<p>No SeatingMap account found.</p>";
    document.getElementById("sectionsTable").innerHTML = "";
    return;
  }

  const seatingMap = decodeSeatingMap(seatingMapAcc.data);
  console.log("Decoded SeatingMap data: " + JSON.stringify(seatingMap, null, 2));

  document.getElementById("seatingCounter").innerHTML =
    `<p><b>Initialized seats:</b> ${seatingMap.total_seats} / ${window.currentEvent.available_tickets}</p>`;

  let sections = [];
  for (let sectionPubkey of seatingMap.sections) {
    const sectionAcc = await connection.getAccountInfo(new solanaWeb3.PublicKey(sectionPubkey));
    if (sectionAcc) {
      const sectionData = decodeSeatingSectionAccount(sectionAcc.data);
      sections.push(sectionData);
    } else {
      console.log("Section account not found: " + sectionPubkey);
    }
  }

  showSeatingSections(sections);
}

function showSeatingSections(sections) {
  const container = document.getElementById("sectionsTable");

  if (sections.length === 0) {
    container.innerHTML = "<p>No seating sections available.</p>";
    return;
  }

  let html = `<table>
    <tr>
      <th>Section Name</th>
      <th>Type</th>
      <th>Rows</th>
      <th>Seats per Row</th>
      <th>Ticket Price</th>
      <th>Seat Preview</th>
      <th>Actions</th>
    </tr>`;

  sections.forEach(sec => {
    const typeStr = sec.section_type === 1 ? "Numbered" : "Standing";
    let previewHTML = `<div class="seat-preview" style="grid-template-columns: repeat(${sec.seats_per_row}, 10px);">`;
    const totalSeats = sec.rows * sec.seats_per_row;

    for (let i = 0; i < totalSeats; i++) {
      const color = sec.seat_status[i] === 0 ? "#8fbc8f" : "#ff7f7f";
      previewHTML += `<div style="background-color:${color};"></div>`;
    }

    previewHTML += `</div>`;

    html += `<tr>
      <td>${sec.section_name}</td>
      <td>${typeStr}</td>
      <td>${sec.rows}</td>
      <td>${sec.seats_per_row}</td>
      <td>${sec.ticket_price}</td>
      <td>${previewHTML}</td>
      <td>
        <button onclick="editSection('${sec.section_name}')">Edit</button>
        <button onclick="deleteSection('${sec.section_name}')">Delete</button>
      </td>
    </tr>`;
  });

  html += `</table>`;
  container.innerHTML = html;
}
