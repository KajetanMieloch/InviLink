async function initConnection() {
    if (!window.phantom || !window.phantom.solana) {
        showErrorAlertwithMSG("Phantom Wallet is required!");
        return;
    }
    provider = window.phantom.solana;
    if (!provider.isConnected) await provider.connect();
    walletPublicKey = provider.publicKey;
    }  