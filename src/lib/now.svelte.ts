let _now = $state(Date.now());

setInterval(() => {
  _now = Date.now();
}, 1000);

export function now(): number {
  return _now;
}
