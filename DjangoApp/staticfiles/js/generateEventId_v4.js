// Helper function for URL-safe base64 encoding – replaces +, / and removes padding (=)
function base64UrlEncode(buffer) {
    let binary = "";
    const bytes = new Uint8Array(buffer);
  
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i]); // convert each byte to a character
    }
  
    let base64 = btoa(binary); // standard base64
    return base64
      .replace(/\+/g, "-") // replace '+' with '-'
      .replace(/\//g, "_") // replace '/' with '_'
      .replace(/=+$/, ""); // remove padding '='
}
  
  // Function that generates event_id consistent with on-chain logic.
  // Uses the seed "339562", combines event name, date and organizer's pubkey bytes,
  // applies SHA‑256 hash and URL-safe base64 encoding (and trims it to 12 characters)
  async function generateEventId(name, eventDate, organizer) {
    const seed = new TextEncoder().encode("339562");
    const nameBytes = new TextEncoder().encode(name);
  
    // Convert eventDate to 8 bytes little-endian
    const eventDateBuffer = new Uint8Array(8);
    new DataView(eventDateBuffer.buffer).setBigUint64(0, BigInt(eventDate), true);
  
    // Make sure organizer is a PublicKey object
    let orgPubkey;
    if (typeof organizer === "string") {
      orgPubkey = new solanaWeb3.PublicKey(organizer);
    } else {
      orgPubkey = organizer;
    }
    const organizerBytes = orgPubkey.toBytes();
  
    // Combine all bytes: seed + name + date + organizer
    const totalLength = seed.length + nameBytes.length + eventDateBuffer.length + organizerBytes.length;
    const combined = new Uint8Array(totalLength);
    let offset = 0;
    combined.set(seed, offset); offset += seed.length;
    combined.set(nameBytes, offset); offset += nameBytes.length;
    combined.set(eventDateBuffer, offset); offset += eventDateBuffer.length;
    combined.set(organizerBytes, offset);
  
    const hashBuffer = await crypto.subtle.digest("SHA-256", combined);
    const hashArray = new Uint8Array(hashBuffer);
    const safeBase64 = base64UrlEncode(hashArray);
    return safeBase64.substring(0, 12);
}
