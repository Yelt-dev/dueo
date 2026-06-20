// Catálogo de marcas (Simple Icons, 3000+) cargado de forma PEREZOSA en un chunk
// aparte (dynamic import) para no inflar el bundle inicial del dashboard. Es
// REACTIVO: cuando termina de cargar, la UI que lo consulta se re-renderiza sola.

import type { IconDef } from './icons';

function slugify(s: string): string {
	return s.toLowerCase().replace(/[^a-z0-9]/g, '');
}

let ready = $state(false);
let started = false;
const map = new Map<string, IconDef>();
const list: IconDef[] = [];

// Arranca la carga una sola vez (llamar al montar la app).
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
	ready = true; // dispara reactividad
}

// Leer `ready` dentro de estos getters crea la dependencia reactiva: quien los use
// en un $derived/plantilla se actualiza cuando el catálogo termina de cargar.
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
