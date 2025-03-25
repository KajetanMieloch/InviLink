async function fetchSolPrice() {
    try {
      const res = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd');
      const data = await res.json();
      return data.solana.usd;
    } catch (err) {
      console.error('Error fetching SOL/USD exchange rate:', err);
    }
}