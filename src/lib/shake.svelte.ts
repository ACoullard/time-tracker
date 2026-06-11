export function createShake(durationMs = 400) {
	let shaking = $state(false);
	return {
		get shaking() { return shaking; },
		trigger() {
			shaking = true;
			setTimeout(() => { shaking = false; }, durationMs);
		},
	};
}
