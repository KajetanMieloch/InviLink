function logMessage(message) {
  const logEl = document.getElementById("logContent");
  if (!logEl) {
    console.warn("Element o id 'logContent' nie został znaleziony.");
    return;
  }
  logEl.textContent += message + "\n";
  console.log(message);
}
