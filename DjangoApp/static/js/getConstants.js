async function getConstants() {
    const res = await fetch('/static/constants/program.json');
    const data = await res.json();
  
    return {
      PROGRAM_ID: new solanaWeb3.PublicKey(data.PROGRAM_ID),
      NETWORK: data.NETWORK
    };
  }
  