const anchor = require("@project-serum/anchor");

async function main() {
  // Używamy providera ustawionego w zmiennych środowiskowych lub z konfiguracji Anchor
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Pobierz referencję do programu (Anchor automatycznie ładuje workspace)
  const program = anchor.workspace.Invilink;

  // Oblicz PDA dla organizers_pool (seed musi być taki sam jak w lib.rs)
  const [organizersPool, bump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("organizers_pool")],
    program.programId
  );
  console.log("Organizers Pool PDA:", organizersPool.toBase58());

  // Wywołaj instrukcję initialize_organizers_pool
  const tx = await program.methods.initializeOrganizersPool().accounts({
    organizersPool: organizersPool,
    payer: provider.wallet.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  }).rpc();
  console.log("Transaction signature:", tx);
}

main().then(() => console.log("Inicjalizacja ukończona")).catch(err => console.error(err));
