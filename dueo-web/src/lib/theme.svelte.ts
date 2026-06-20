// Estado de tema reactivo (Svelte 5 runes en un .svelte.ts).
// El parpadeo lo evita el script inline de app.html; aquí solo sincronizamos
// el estado y persistimos los cambios.

type Mode = 'light' | 'dark';

function createTheme() {
	let mode = $state<Mode>('dark');

	return {
		get mode() {
			return mode;
		},
		// Lee el tema que ya dejó puesto el script de app.html.
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
