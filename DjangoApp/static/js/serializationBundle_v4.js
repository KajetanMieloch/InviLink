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

  // Funkcja dekodująca rejestr eventów – teraz czyta dynamiczny wektor:
  function decodeRegistry(data) {
    let offset = 8; // pomijamy 8-bajtowy discriminator
    const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
    const eventCount = dv.getUint32(offset, true);
    offset += 4;
    // Odczytujemy długość wektora (u32)
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

 // Dekodowanie konta eventu zgodnie z formatem Anchor, uwzględniające nowe pole event_date
 function decodeEvent(data) {
  let offset = 8; // pomijamy 8-bajtowy discriminator
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