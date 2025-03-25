// Serializes a string: [4 bytes little-endian length + UTF-8 bytes]
function serializeString(str) {
    const encoder = new TextEncoder();
    const strBytes = encoder.encode(str); // Convert string to UTF-8 byte array
    const lengthBytes = new Uint8Array(4); // 4 bytes for length
    new DataView(lengthBytes.buffer).setUint32(0, strBytes.length, true); // Write length as little-endian u32
    const result = new Uint8Array(4 + strBytes.length); // Total buffer = 4 bytes (length) + string bytes
    result.set(lengthBytes, 0); // Set length at offset 0
    result.set(strBytes, 4); // Set UTF-8 string bytes at offset 4
    return result;
  }
  
  // Serializes a BN u64 into 8 little-endian bytes
  function serializeU64(bnValue) {
    return bnValue.toArrayLike(Uint8Array, 'le', 8);
  }
  
  // Serializes arguments for create_event_seating
  // Arguments: event_id (string), name (string), event_date (i64), available_tickets (u64)
  function serializeCreateEventSeatingArgs({ event_id, name, event_date, available_tickets }) {
    const eventIdBytes = serializeString(event_id);
    const nameBytes = serializeString(name);
    // event_date is serialized as i64 (UNIX timestamp in seconds)
    const eventDateBytes = serializeU64(new BN(event_date.toString()));
    const availableBytes = serializeU64(available_tickets);
  
    const totalLen =
      eventIdBytes.length +
      nameBytes.length +
      eventDateBytes.length +
      availableBytes.length;
  
    const buffer = new Uint8Array(totalLen);
    let offset = 0;
    buffer.set(eventIdBytes, offset); offset += eventIdBytes.length;
    buffer.set(nameBytes, offset); offset += nameBytes.length;
    buffer.set(eventDateBytes, offset); offset += eventDateBytes.length;
    buffer.set(availableBytes, offset);
  
    return buffer;
  }

 // Decode event registry – reads a dynamic vector of Pubkeys
function decodeRegistry(data) {
  let offset = 8; // skip 8-byte discriminator
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const eventCount = dv.getUint32(offset, true);
  offset += 4;

  // Read vector length (u32)
  const vecLen = dv.getUint32(offset, true);
  offset += 4;
  const events = [];

  for (let i = 0; i < vecLen; i++) {
    const pubkeyBytes = data.slice(offset, offset + 32);
    const pubkey = new solanaWeb3.PublicKey(pubkeyBytes).toBase58();
    offset += 32;
    events.push(pubkey);
  }

  return { eventCount, events };
}

// Decode event account according to Anchor layout – includes event_date
function decodeEvent(data) {
  let offset = 8; // skip 8-byte discriminator
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);

  function readString() {
    if (offset + 4 > data.byteLength) return "";
    const len = dv.getUint32(offset, true);
    offset += 4;
    if (offset + len > data.byteLength) return "";
    const strBytes = data.slice(offset, offset + len);
    offset += len;
    return new TextDecoder().decode(strBytes);
  }

  const event_id = readString();

  let organizer = "";
  if (offset + 32 <= data.byteLength) {
    const orgBytes = data.slice(offset, offset + 32);
    organizer = new solanaWeb3.PublicKey(orgBytes).toBase58();
    offset += 32;
  }

  const name = readString();

  let event_date = 0;
  if (offset + 8 <= data.byteLength) {
    event_date = Number(dv.getBigUint64(offset, true));
    offset += 8;
  }

  let available_tickets = "0";
  if (offset + 8 <= data.byteLength) {
    available_tickets = dv.getBigUint64(offset, true).toString();
    offset += 8;
  }

  let sold_tickets = "0";
  if (offset + 8 <= data.byteLength) {
    sold_tickets = dv.getBigUint64(offset, true).toString();
    offset += 8;
  }

  let seating_type = 0;
  if (offset + 1 <= data.byteLength) {
    seating_type = dv.getUint8(offset);
    offset += 1;
  }

  let active = false;
  if (offset + 1 <= data.byteLength) {
    active = dv.getUint8(offset) !== 0;
    offset += 1;
  }

  return {
    event_id,
    organizer,
    name,
    event_date,
    available_tickets,
    sold_tickets,
    seating_type,
    active,
  };
}

// Extended decode function for event account, including validators
function decodeEventWithValidators(data) {
  let offset = 8;
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);

  function readString() {
    if (offset + 4 > data.byteLength) return "";
    const len = dv.getUint32(offset, true);
    offset += 4;
    if (offset + len > data.byteLength) return "";
    const strBytes = data.slice(offset, offset + len);
    offset += len;
    return new TextDecoder().decode(strBytes);
  }

  const event_id = readString();

  let organizer = "";
  if (offset + 32 <= data.byteLength) {
    const orgBytes = data.slice(offset, offset + 32);
    organizer = new solanaWeb3.PublicKey(orgBytes).toBase58();
    offset += 32;
  }

  const name = readString();

  let event_date = 0;
  if (offset + 8 <= data.byteLength) {
    event_date = Number(dv.getBigUint64(offset, true));
    offset += 8;
  }

  let available_tickets = "0";
  if (offset + 8 <= data.byteLength) {
    available_tickets = dv.getBigUint64(offset, true).toString();
    offset += 8;
  }

  let sold_tickets = "0";
  if (offset + 8 <= data.byteLength) {
    sold_tickets = dv.getBigUint64(offset, true).toString();
    offset += 8;
  }

  let seating_type = 0;
  if (offset + 1 <= data.byteLength) {
    seating_type = dv.getUint8(offset);
    offset += 1;
  }

  let active = false;
  if (offset + 1 <= data.byteLength) {
    active = dv.getUint8(offset) !== 0;
    offset += 1;
  }

  // Read validators length (u32) and list of Pubkeys
  let validators = [];
  if (offset + 4 <= data.byteLength) {
    const validatorsLen = dv.getUint32(offset, true);
    offset += 4;
    for (let i = 0; i < validatorsLen; i++) {
      if (offset + 32 > data.byteLength) break;
      const validatorBytes = data.slice(offset, offset + 32);
      const validator = new solanaWeb3.PublicKey(validatorBytes).toBase58();
      validators.push(validator);
      offset += 32;
    }
  }

  return {
    event_id,
    organizer,
    name,
    event_date,
    available_tickets,
    sold_tickets,
    seating_type,
    active,
    validators
  };
}

// Helper function for serializing optional u8 values
function serializeOptionU8(value) {
  if (value === null || isNaN(value)) {
    return new Uint8Array([0]);
  } else {
    return new Uint8Array([1, value]);
  }
}

// Helper function for serializing optional u64 values
function serializeOptionU64(value) {
  if (value === null || isNaN(value)) {
    return new Uint8Array([0]);
  } else {
    const flag = new Uint8Array([1]);
    const valueBytes = serializeU64(new BN(value));
    const combined = new Uint8Array(1 + valueBytes.length);
    combined.set(flag, 0);
    combined.set(valueBytes, 1);
    return combined;
  }
}

// Serialize arguments for initialize_seating_section:
// (section_name: string, section_type: u8, rows: u8, seats_per_row: u8, ticket_price: u64)
function serializeInitializeSeatingSectionArgs({ section_name, section_type, rows, seats_per_row, ticket_price }) {
  const sectionNameBytes = serializeString(section_name);
  const sectionTypeByte = new Uint8Array([section_type]);
  const rowsByte = new Uint8Array([rows]);
  const seatsByte = new Uint8Array([seats_per_row]);
  const ticketPriceBytes = serializeU64(new BN(ticket_price));

  const totalLen = sectionNameBytes.length + sectionTypeByte.length + rowsByte.length + seatsByte.length + ticketPriceBytes.length;
  const buffer = new Uint8Array(totalLen);
  let offset = 0;
  buffer.set(sectionNameBytes, offset); offset += sectionNameBytes.length;
  buffer.set(sectionTypeByte, offset); offset += sectionTypeByte.length;
  buffer.set(rowsByte, offset); offset += rowsByte.length;
  buffer.set(seatsByte, offset); offset += seatsByte.length;
  buffer.set(ticketPriceBytes, offset);

  return buffer;
}

// Decode SeatingMap – expects: event_id, vec<Pubkey>, total_seats
function decodeSeatingMap(data) {
  let offset = 8; // skip 8-byte discriminator
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);

  const eventIdLen = dv.getUint32(offset, true); 
  offset += 4;
  const eventIdBytes = data.slice(offset, offset + eventIdLen); 
  offset += eventIdLen;
  const event_id = new TextDecoder().decode(eventIdBytes);

  const organizerBytes = data.slice(offset, offset + 32);
  offset += 32;
  const organizer = new solanaWeb3.PublicKey(organizerBytes).toBase58();

  const active = dv.getUint8(offset) !== 0;
  offset += 1;

  const vecLen = dv.getUint32(offset, true);
  offset += 4;

  let sections = [];
  for (let i = 0; i < vecLen; i++) {
    const keyBytes = data.slice(offset, offset + 32);
    sections.push(new solanaWeb3.PublicKey(keyBytes).toBase58());
    offset += 32;
  }

  const total_seats = dv.getBigUint64(offset, true);
  offset += 8;

  return {
    event_id,
    organizer,
    active,
    sections,
    total_seats: total_seats.toString()
  };
}

// Decode SeatingSectionAccount
function decodeSeatingSectionAccount(data) {
  let offset = 8; // skip discriminator
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);

  const eventIdLen = dv.getUint32(offset, true); offset += 4;
  const eventIdBytes = data.slice(offset, offset + eventIdLen); offset += eventIdLen;
  const event_id = new TextDecoder().decode(eventIdBytes);

  const sectionNameLen = dv.getUint32(offset, true); offset += 4;
  const sectionNameBytes = data.slice(offset, offset + sectionNameLen); offset += sectionNameLen;
  const section_name = new TextDecoder().decode(sectionNameBytes);

  const section_type = dv.getUint8(offset); offset += 1;
  const rows = dv.getUint8(offset); offset += 1;
  const seats_per_row = dv.getUint8(offset); offset += 1;
  const ticket_price = dv.getBigUint64(offset, true); offset += 8;

  const vecLen = dv.getUint32(offset, true); offset += 4;
  let seat_status = [];
  for (let i = 0; i < vecLen; i++) {
    seat_status.push(dv.getUint8(offset));
    offset += 1;
  }

  return {
    event_id,
    section_name,
    section_type,
    rows,
    seats_per_row,
    ticket_price: ticket_price.toString(),
    seat_status
  };
}

function encodeOptionString(str) {
  if (!str || str.trim() === "") {
    return new Uint8Array([0]); // opcjon "None"
  }
  const encoder = new TextEncoder();
  const strBytes = encoder.encode(str);
  const len = strBytes.length;
  
  const result = new Uint8Array(1 + 4 + len);
  result[0] = 1; // opcjon "Some"
  result[1] = len & 0xff;
  result[2] = (len >> 8) & 0xff;
  result[3] = (len >> 16) & 0xff;
  result[4] = (len >> 24) & 0xff;
  result.set(strBytes, 5);
  return result;
}

function encodeOptionU8(value) {
  if (value === null || isNaN(value)) {
    return new Uint8Array([0]); // opcjon "None"
  }
  return new Uint8Array([1, value & 0xff]); // opcjon "Some"
}

function encodeOptionU64(value) {
  if (value === null || isNaN(value)) {
    return new Uint8Array([0]); // Option::None
  }

  const bn = new BN(value); // using bn.js
  const encoded = bn.toArray('le', 8); // 8 bytes little-endian
  const result = new Uint8Array(1 + 8);
  result[0] = 1; // Option::Some
  result.set(encoded, 1);
  return result;
}

function serializeU8(val) {
  return new Uint8Array([val]);
}


function buildValidateTicketData(eventId, section, row, seat) {
  const VALIDATE_TICKET_DISCRIMINATOR = new Uint8Array([222, 125, 246, 215, 10, 163, 159, 200]);
    const eventIdBytes = serializeString(eventId);
    const sectionBytes = serializeString(section);
    const rowBytes = serializeU8(row);
    const seatBytes = serializeU8(seat);
    const totalLength = VALIDATE_TICKET_DISCRIMINATOR.length + eventIdBytes.length + sectionBytes.length + rowBytes.length + seatBytes.length;
    const data = new Uint8Array(totalLength);
    let offset = 0;
    data.set(VALIDATE_TICKET_DISCRIMINATOR, offset); offset += VALIDATE_TICKET_DISCRIMINATOR.length;
    data.set(eventIdBytes, offset); offset += eventIdBytes.length;
    data.set(sectionBytes, offset); offset += sectionBytes.length;
    data.set(rowBytes, offset); offset += rowBytes.length;
    data.set(seatBytes, offset);
    return data;
  }

  function decodeEventNFT(data) {
    let offset = 8;
    const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
    const eventIdLen = dv.getUint32(offset, true); offset += 4;
    const eventIdBytes = data.slice(offset, offset + eventIdLen); offset += eventIdLen;
    const event_id = new TextDecoder().decode(eventIdBytes);
    const organizerBytes = data.slice(offset, offset + 32); offset += 32;
    const organizer = new solanaWeb3.PublicKey(organizerBytes).toBase58();
    const nameLen = dv.getUint32(offset, true); offset += 4;
    const nameBytes = data.slice(offset, offset + nameLen); offset += nameLen;
    const name = new TextDecoder().decode(nameBytes);
    let event_date = 0;
    if (offset + 8 <= data.byteLength) {
      event_date = Number(dv.getBigUint64(offset, true));
      offset += 8;
    }
    let available_tickets = "0";
    if (offset + 8 <= data.byteLength) {
      available_tickets = dv.getBigUint64(offset, true).toString();
      offset += 8;
    }
    let sold_tickets = "0";
    if (offset + 8 <= data.byteLength) {
      sold_tickets = dv.getBigUint64(offset, true).toString();
      offset += 8;
    }
    let seating_type = 0;
    if (offset + 1 <= data.byteLength) {
      seating_type = dv.getUint8(offset);
      offset += 1;
    }
    let active = false;
    if (offset + 1 <= data.byteLength) {
      active = dv.getUint8(offset) !== 0;
      offset += 1;
    }
    return {
      event_id,
      organizer,
      name,
      event_date,
      available_tickets,
      sold_tickets,
      seating_type,
      active
    };
  }


  // Własna funkcja deserializująca metadane NFT – zakładamy stały format pól
function customDeserializeMetadata(buffer) {
  let offset = 8; // pomijamy 8 bajtów dyskryminatora
  const dv = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
  
  const key = dv.getUint8(offset);
  offset += 1;
  
  const updateAuthorityBytes = buffer.slice(offset, offset + 32);
  const updateAuthority = new solanaWeb3.PublicKey(updateAuthorityBytes).toBase58();
  offset += 32;
  
  const mintBytes = buffer.slice(offset, offset + 32);
  const mint = new solanaWeb3.PublicKey(mintBytes).toBase58();
  offset += 32;
  
  const nameBytes = buffer.slice(offset, offset + 32);
  let name = new TextDecoder().decode(nameBytes);
  name = name.replace(/\0/g, "").trim();
  offset += 32;
  
  const symbolBytes = buffer.slice(offset, offset + 10);
  let symbol = new TextDecoder().decode(symbolBytes);
  symbol = symbol.replace(/\0/g, "").trim();
  offset += 10;
  
  const uriBytes = buffer.slice(offset, offset + 200);
  let uri = new TextDecoder().decode(uriBytes);
  uri = uri.replace(/\0/g, "").trim();
  offset += 200;
  
  return { key, updateAuthority, mint, name, symbol, uri };
}


// Funkcja budująca dane dla instrukcji activate_ticket:
// Format: [discriminator (8 bajtów)] || [serializeString(event_id)] || [serializeString(section)] || [serializeU8(row)] || [serializeU8(seat)]
function buildActivateTicketData(eventId, section, row, seat) {
  
  const ACTIVATE_TICKET_DISCRIMINATOR = new Uint8Array([110, 8, 92, 34, 61, 23, 0, 151]);

  const eventIdBytes = serializeString(eventId);
  const sectionBytes = serializeString(section);
  const rowBytes = serializeU8(row);
  const seatBytes = serializeU8(seat);
  const totalLength = ACTIVATE_TICKET_DISCRIMINATOR.length + eventIdBytes.length + sectionBytes.length + rowBytes.length + seatBytes.length;
  const data = new Uint8Array(totalLength);
  let offset = 0;
  data.set(ACTIVATE_TICKET_DISCRIMINATOR, offset);
  offset += ACTIVATE_TICKET_DISCRIMINATOR.length;
  data.set(eventIdBytes, offset); offset += eventIdBytes.length;
  data.set(sectionBytes, offset); offset += sectionBytes.length;
  data.set(rowBytes, offset); offset += rowBytes.length;
  data.set(seatBytes, offset);
  return data;
}