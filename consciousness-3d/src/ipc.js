let lastDirection = null;

export function sendInput(direction) {
  if (direction === lastDirection) return; // debounce identical direction
  lastDirection = direction;
  window.api.sendToPython({ type: 'input', direction });
}

export function sendConfig(obj) {
  window.api.sendToPython({ type: 'config', ...obj });
}
