

async function fetchSolPrice() {
    let exchangeRate = 0;
  try {
    const res = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=solana&vs_currencies=usd');
    const data = await res.json();
    exchangeRate = data.solana.usd;
  } catch (err) {
    console.error('Error fetching SOL/USD exchange rate:', err);
  }
}

function syncPrices(e) {
    let exchangeRate = 0;
  const solInput = document.getElementById('ticketPriceInput');
  const usdInput = document.getElementById('ticketPriceUSDInput');

  if (!exchangeRate) return;

  if (e.target.id === 'ticketPriceInput') {
    const sol = parseFloat(solInput.value);
    if (!isNaN(sol)) usdInput.value = ((sol / 1e9) * exchangeRate).toFixed(2);
  } else {
    const usd = parseFloat(usdInput.value);
    if (!isNaN(usd)) solInput.value = Math.round((usd / exchangeRate) * 1e9);
  }
}