const { PublicKey } = require("@solana/web3.js");

const programId = new PublicKey("FEJSabV32RFGaVfgFt6kor74kEbYFkCaXSznEZi8zQPy"); // Twój Program ID
const seed = Buffer.from("mint_authority");

(async () => {
  const [pda, bump] = await PublicKey.findProgramAddress([seed], programId);
  console.log("PDA Address:", pda.toBase58());
  console.log("Bump Seed:", bump);
})();
