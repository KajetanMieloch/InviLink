import nacl from 'tweetnacl';
import { encode, decode } from 'bs58';

// Generowanie kluczy
const keyPair = nacl.box.keyPair();
const dappEncryptionPublicKey = encode(keyPair.publicKey);
const dappEncryptionPrivateKey = keyPair.secretKey;

export { dappEncryptionPublicKey, dappEncryptionPrivateKey };
