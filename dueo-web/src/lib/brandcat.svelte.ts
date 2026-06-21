// Brand catalog (Simple Icons, 3000+) loaded LAZILY in a separate chunk
// (dynamic import) to keep it out of the dashboard's initial bundle. It is
// REACTIVE: once loading finishes, any UI reading it re-renders automatically.

import type { IconDef } from './icons';

function slugify(s: string): string {
	return s.toLowerCase().replace(/[^a-z0-9]/g, '');
}

let ready = $state(false);
let started = false;
const map = new Map<string, IconDef>();
const list: IconDef[] = [];

// Kicks off loading once (call on app mount).
export async function ensureBrands(): Promise<void> {
	if (started) return;
	started = true;
	const simple = (await import('simple-icons')) as Record<string, unknown>;
	for (const v of Object.values(simple)) {
		const s = v as { path?: string; title?: string; hex?: string; slug?: string };
		if (s && typeof s.path === 'string' && s.title && s.hex) {
			const id = `si:${s.slug ?? slugify(s.title)}`;
			if (!map.has(id)) {
				const def: IconDef = { id, kind: 'si', label: s.title, path: s.path, color: '#' + s.hex };
				map.set(id, def);
				list.push(def);
			}
		}
	}
	ready = true; // trigger reactivity
}

// Reading `ready` inside these getters creates the reactive dependency: anyone
// using them in a $derived/template updates when the catalog finishes loading.
export function brandsReady(): boolean {
	return ready;
}
export function allBrands(): IconDef[] {
	void ready;
	return list;
}
export function brandById(id: string): IconDef | null {
	void ready;
	return map.get(id) ?? null;
}
