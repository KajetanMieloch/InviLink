<!DOCTYPE html>
<html>
  <head>
    <title>Mint NFT to My Own ATA</title>
  </head>
  <body>
    <button onclick="mintTokenToMyATA()">Mint My Token</button>

    <!-- 1) Solana Web3 -->
    <script src="https://unpkg.com/@solana/web3.js@latest/lib/index.iife.js"></script>

    <script>
      const solanaWeb3 = window.solanaWeb3;

      // Program ID Twojego smart kontraktu
      const PROGRAM_ID = new solanaWeb3.PublicKey("Hh9NSEH8cZv8Vhq5PhN88CKBndPQnDCzc513V9B1xeZH");

      // Adres Mintu, który ma decimals=0
      const MINT_ADDRESS = new solanaWeb3.PublicKey("Cq42RxfDUetfwfESwH9SXMvGfgj3KvHqT3wGLksnsuyN");

      // Token Program (SPL)
      const TOKEN_PROGRAM_ID = new solanaWeb3.PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

      // Associated Token Program
      const ASSOCIATED_TOKEN_PROGRAM_ID = new solanaWeb3.PublicKey(
        "ATokenGPvRzJJrto1k6y6hvj83Rr1Pc2hFMG6VZ9PaHT"
      );

      // Discriminator dla "mint_token" wg IDL ([172, 137, 183, 14, 207, 110, 234, 56])
      const MINT_TOKEN_DISCRIMINATOR = new Uint8Array([172, 137, 183, 14, 207, 110, 234, 56]);

      // 8-bajtowa reprezentacja "1" (u64, little-endian)
      const AMOUNT_1 = new Uint8Array([1, 0, 0, 0, 0, 0, 0, 0]);

      /**
       *  Tworzy instrukcję do utworzenia ATA, jeśli on nie istnieje
       *  (typowa instrukcja Associated Token Programu).
       *
       *  Payer płaci za transakcję. Owner = publicKey właściciela konta.
       */
      function createAssociatedTokenAccountInstruction(
        payerPubkey,
        associatedTokenPubkey,
        ownerPubkey,
        mintPubkey
      ) {
        const keys = [
          { pubkey: payerPubkey,          isSigner: true,  isWritable: true }, // Płaci za rent
          { pubkey: associatedTokenPubkey, isSigner: false, isWritable: true },
          { pubkey: ownerPubkey,           isSigner: false, isWritable: false },
          { pubkey: mintPubkey,            isSigner: false, isWritable: false },
          { pubkey: solanaWeb3.SystemProgram.programId,     isSigner: false, isWritable: false },
          { pubkey: TOKEN_PROGRAM_ID,      isSigner: false, isWritable: false },
          { pubkey: solanaWeb3.SYSVAR_RENT_PUBKEY,          isSigner: false, isWritable: false },
        ];

        return new solanaWeb3.TransactionInstruction({
          keys,
          programId: ASSOCIATED_TOKEN_PROGRAM_ID,
          data: Buffer.alloc(0), // Associated Token Program doesn't require data
        });
      }

      async function mintTokenToMyATA() {
        // 1) Wykryj i połącz Phantom
        const provider = getPhantomProvider();
        if (!provider) return;
        if (!provider.publicKey) {
          await provider.connect(); 
        }
        const userPubkey = provider.publicKey;

        // 2) Połącz się z Devnet
        const connection = new solanaWeb3.Connection(
          solanaWeb3.clusterApiUrl("devnet"),
          "confirmed"
        );

        // 3) Oblicz, jaki powinien być ATA dla (userPubkey, MINT_ADDRESS)
        const [ataPubkey] = await solanaWeb3.PublicKey.findProgramAddress(
          [
            userPubkey.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            MINT_ADDRESS.toBuffer(),
          ],
          ASSOCIATED_TOKEN_PROGRAM_ID
        );

        // 4) Sprawdź, czy ATA już istnieje
        const ataAccountInfo = await connection.getAccountInfo(ataPubkey);

        // 5) Zbuduj transakcję
        const transaction = new solanaWeb3.Transaction();

        // 5a) Jeśli ATA nie istnieje -> dołącz instrukcję utworzenia ATA
        if (!ataAccountInfo) {
          const createATAIx = createAssociatedTokenAccountInstruction(
            userPubkey,  // payer
            ataPubkey,   // to-be-created ATA
            userPubkey,  // owner
            MINT_ADDRESS // which mint
          );
          transaction.add(createATAIx);
        }

        // 5b) Dodaj instrukcję mintowania z Twojego Anchor SC
        // Data = [discriminator (8b), amount (8b)]
        const data = new Uint8Array([
          ...MINT_TOKEN_DISCRIMINATOR,
          ...AMOUNT_1,
        ]);

        // Zgodnie z IDL: 
        //   mint -> MINT_ADDRESS
        //   to   -> ataPubkey
        //   payer -> userPubkey
        //   token_program -> TOKEN_PROGRAM_ID
        const mintIx = new solanaWeb3.TransactionInstruction({
          keys: [
            { pubkey: MINT_ADDRESS,     isSigner: false, isWritable: true },
            { pubkey: ataPubkey,        isSigner: false, isWritable: true },
            { pubkey: userPubkey,       isSigner: true,  isWritable: true },
            { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
          ],
          programId: PROGRAM_ID,
          data: data,
        });
        transaction.add(mintIx);

        // 5c) Ustaw feePayer i blockhash
        const { blockhash } = await connection.getLatestBlockhash();
        transaction.recentBlockhash = blockhash;
        transaction.feePayer = userPubkey;

        // 6) Opcjonalna symulacja
        const sim = await connection.simulateTransaction(transaction);
        if (sim.value.err) {
          console.warn("Simulation logs:", sim.value.logs);
          throw new Error("Simulation failed. Check logs.");
        }

        // 7) signAndSendTransaction w Phantom
        try {
          const { signature } = await provider.signAndSendTransaction(transaction);
          await connection.confirmTransaction(signature);
          alert(`Minted! Check Explorer:\n${signature}`);
        } catch (err) {
          console.error("Mint failed:", err);
          alert(`Mint failed: ${err.message}`);
        }
      }

      function getPhantomProvider() {
        if ("phantom" in window && window.phantom.solana) {
          const prov = window.phantom.solana;
          if (prov.isPhantom) return prov;
        }
        alert("Phantom Wallet is required!");
        return null;
      }
    </script>
  </body>
</html>
