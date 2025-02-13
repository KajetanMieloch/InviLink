const {
    Connection,
    Keypair,
    SystemProgram,
    Transaction,
    sendAndConfirmTransaction,
    PublicKey
} = require("@solana/web3.js");
const fs = require("fs");
const path = "organizers_pool.json";

(async () => {
    const connection = new Connection("https://api.devnet.solana.com", "confirmed");

    let organizersPool;
    if (fs.existsSync(path)) {
        console.log("Plik organizers_pool.json istnieje, generuję nowe konto...");
    }

    // Generowanie nowego Keypair dla organizersPool
    organizersPool = Keypair.generate();
    console.log("Nowe konto organizersPool:", organizersPool.publicKey.toBase58());

    // Zapisujemy nowy klucz do pliku
    fs.writeFileSync(path, JSON.stringify(Array.from(organizersPool.secretKey)));

    // Obliczenie wymaganych lamportów na wynajem
    const space = 500; // Rozmiar w bajtach
    const lamports = await connection.getMinimumBalanceForRentExemption(space);

    // Program ID Twojego kontraktu
    const programId = new PublicKey("DFjEJhNS8wMAvV3gFbVf2JiCkbsXBt9uuZBP2ZMotXey");

    // Payer – używamy klucza z ~/.config/solana/id.json
    const payerData = JSON.parse(fs.readFileSync("/home/alternator/.config/solana/id.json"));
    const payer = Keypair.fromSecretKey(new Uint8Array(payerData));

    // Tworzenie nowego konta dla organizersPool
    const transaction = new Transaction().add(
        SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: organizersPool.publicKey,
            lamports,
            space,
            programId,
        })
    );

    // Wysyłanie transakcji
    const signature = await sendAndConfirmTransaction(connection, transaction, [payer, organizersPool]);
    console.log("OrganizersPool utworzone. Signature:", signature);
})();
