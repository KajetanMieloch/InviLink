function logMessage(message) {
    const logEl = document.getElementById("log");
    logEl.textContent += message + "\n";
    console.log(message);
  }
