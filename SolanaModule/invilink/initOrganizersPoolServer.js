const {
    Connection,
    Keypair,
    SystemProgram,
    Transaction,
    sendAndConfirmTransaction,
    PublicKey
} = require("@solana/web3.js");
const fs = require("fs");

const NETWORK_URL = "https://api.devnet.solana.com";
const PROGRAM_ID = new PublicKey("DqZf5dE14GCM541qRBNipykFFHDMe2DKxshWk2Q4McMU");
const DISCRIMINATOR = new Uint8Array([175, 175, 109, 31, 13, 152, 155, 237]); // 8-bajtowy discriminator dla `initialize`

const PAYER_KEY_PATH = "/home/alternator/.config/solana/id.json";

(async () => {
    console.log("🔵 Inicjalizacja `organizers_pool`...");

    const connection = new Connection(NETWORK_URL, "confirmed");

    if (!fs.existsSync(PAYER_KEY_PATH)) {
        console.error("❌ ERROR: Nie znaleziono pliku klucza:", PAYER_KEY_PATH);
        return;
    }
    const payerData = JSON.parse(fs.readFileSync(PAYER_KEY_PATH));
    const payer = Keypair.fromSecretKey(new Uint8Array(payerData));

    console.log("✅ Wczytano klucz Payera:", payer.publicKey.toBase58());

    const organizersPool = Keypair.generate();
    console.log("✅ Wygenerowano `organizers_pool`:", organizersPool.publicKey.toBase58());

    const space = 1000; // Zapas miejsca - więcej niż wymagane
    const lamports = await connection.getMinimumBalanceForRentExemption(space);

    const transaction = new Transaction().add(
        SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: organizersPool.publicKey,
            lamports,
            space,
            programId: PROGRAM_ID,
        })
    );

    console.log("🔄 Wysyłanie transakcji...");

    try {
        const signature = await sendAndConfirmTransaction(connection, transaction, [payer, organizersPool]);
        console.log("✅ `organizers_pool` utworzone! Signature:", signature);

        fs.writeFileSync("organizers_pool.json", JSON.stringify(Array.from(organizersPool.secretKey)));
        console.log("✅ Klucz zapisany do organizers_pool.json");
    } catch (error) {
        console.error("❌ Błąd przy wysyłaniu transakcji:", error);
    }
})();