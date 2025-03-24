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
  // Arguments: event_id (string), name (string), event_date (i64), ticket_price (u64), available_tickets (u64)
  function serializeCreateEventSeatingArgs({ event_id, name, event_date, ticket_price, available_tickets }) {
    const eventIdBytes = serializeString(event_id);
    const nameBytes = serializeString(name);
    // event_date is serialized as i64 (UNIX timestamp in seconds)
    const eventDateBytes = serializeU64(new BN(event_date.toString()));
    const ticketPriceBytes = serializeU64(ticket_price);
    const availableBytes = serializeU64(available_tickets);
  
    const totalLen =
      eventIdBytes.length +
      nameBytes.length +
      eventDateBytes.length +
      ticketPriceBytes.length +
      availableBytes.length;
  
    const buffer = new Uint8Array(totalLen);
    let offset = 0;
    buffer.set(eventIdBytes, offset); offset += eventIdBytes.length;
    buffer.set(nameBytes, offset); offset += nameBytes.length;
    buffer.set(eventDateBytes, offset); offset += eventDateBytes.length;
    buffer.set(ticketPriceBytes, offset); offset += ticketPriceBytes.length;
    buffer.set(availableBytes, offset);
  
    return buffer;
  }
  