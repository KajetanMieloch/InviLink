async function getConstants() {
    const res = await fetch('/static/constants/program.json');
    return await res.json();
  }
  