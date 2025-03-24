async function initConnection() {
    if (!window.phantom || !window.phantom.solana) {
        alert("Phantom Wallet is required wymagany!");
        return;
    }
    provider = window.phantom.solana;
    if (!provider.isConnected) await provider.connect();
    walletPublicKey = provider.publicKey;
    }  