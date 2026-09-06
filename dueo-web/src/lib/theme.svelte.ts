// Reactive theme state (Svelte 5 runes in a .svelte.ts).
// The inline script in app.html prevents the flash; here we only sync the
// state and persist changes.

type Mode = 'light' | 'dark';

// startViewTransition isn't in every lib.dom yet.
type VTDocument = Document & {
	startViewTransition?: (cb: () => void) => { finished: Promise<void> };
};

const SWITCH_MS = 320;

function prefersReducedMotion(): boolean {
	return window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;
}

function createTheme() {
	let mode = $state<Mode>('dark');
	let switchTimer: ReturnType<typeof setTimeout>;

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
			const next: Mode = mode === 'dark' ? 'light' : 'dark';
			const apply = () => {
				mode = next;
				document.documentElement.dataset.theme = next;
				try {
					localStorage.setItem('theme', next);
				} catch {
					// localStorage unavailable (private mode): the theme still applies.
				}
			};

			if (prefersReducedMotion()) return apply();

			// Best case: the browser cross-fades a snapshot of the whole page, which
			// covers what CSS can't interpolate (the body gradient, the acrylic noise).
			const doc = document as VTDocument;
			if (typeof doc.startViewTransition === 'function') {
				doc.startViewTransition(apply);
				return;
			}

			// Fallback: ease every colour for the length of the switch only, so the
			// rest of the time components keep their own transitions untouched.
			const root = document.documentElement;
			root.classList.add('theme-switching');
			clearTimeout(switchTimer);
			switchTimer = setTimeout(() => root.classList.remove('theme-switching'), SWITCH_MS);
			apply();
		}
	};
}

export const theme = createTheme();
