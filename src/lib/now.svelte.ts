let _now = $state(Date.now());
let intervalId: ReturnType<typeof setInterval>;

function startTicking() {
  intervalId = setInterval(() => {
    _now = Date.now();
  }, 1000);
}

startTicking();

export function now(): number {
  return _now;
}

export function bumpNow() {
  _now = Date.now();
  clearInterval(intervalId);
  startTicking();
}
