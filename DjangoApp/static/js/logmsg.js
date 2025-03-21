function logMessage(message) {
    const logEl = document.getElementById("logContent");
    logEl.textContent += message + "\n";
    console.log(message);
  }
