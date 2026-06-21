// Reusable loading toggle: wraps an async action so the button shows a pending
// state without each handler repeating the `flag = true; try { … } finally
// { flag = false }` boilerplate. A section can hold several (e.g. save + test).
export function busy() {
	let on = $state(false);
	return {
		get on() {
			return on;
		},
		async run<T>(fn: () => Promise<T>): Promise<T> {
			on = true;
			try {
				return await fn();
			} finally {
				on = false;
			}
		}
	};
}
