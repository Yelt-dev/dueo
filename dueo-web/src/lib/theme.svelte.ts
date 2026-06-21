// Reactive theme state (Svelte 5 runes in a .svelte.ts).
// The inline script in app.html prevents the flash; here we only sync the
// state and persist changes.

type Mode = 'light' | 'dark';

function createTheme() {
	let mode = $state<Mode>('dark');

	return {
		get mode() {
			return mode;
		},
		// Read the theme already set by the app.html script.
		init() {
			const current = document.documentElement.dataset.theme;
			mode = current === 'light' ? 'light' : 'dark';
		},
		toggle() {
			mode = mode === 'dark' ? 'light' : 'dark';
			document.documentElement.dataset.theme = mode;
			localStorage.setItem('theme', mode);
		}
	};
}

export const theme = createTheme();
