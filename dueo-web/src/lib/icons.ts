// Icons for the form: Lucide (generic, static, lightweight) + Simple Icons
// (brands, loaded LAZILY from brandcat to keep the bundle small).
// Color comes from the DUEO HUES palette (the logo ramp).

import {
	Tv,
	Music,
	Cloud,
	Gamepad2,
	Dumbbell,
	BookOpen,
	Car,
	House,
	Briefcase,
	CreditCard,
	Newspaper,
	Smartphone,
	Globe,
	Server,
	Zap,
	Heart,
	ShoppingBag,
	GraduationCap,
	Film,
	Wifi,
	Coffee,
	Camera,
	Code,
	Mail,
	Database,
	Shield,
	Headphones,
	Palette
} from '@lucide/svelte';
import type { Component } from 'svelte';
import { brandById, allBrands, brandsReady } from './brandcat.svelte';
import { brandFor, type Brand } from './brands';

export type IconDef =
	| { id: string; kind: 'lu'; label: string; comp: Component }
	| { id: string; kind: 'si'; label: string; path: string; color: string };

const lu = (id: string, label: string, comp: Component): IconDef => ({
	id: `lu:${id}`,
	kind: 'lu',
	label,
	comp
});

// Generic icons (Lucide) by expense type.
export const LUCIDE_ICONS: IconDef[] = [
	lu('tv', 'Streaming', Tv),
	lu('film', 'Cine', Film),
	lu('music', 'Música', Music),
	lu('headphones', 'Audio', Headphones),
	lu('gamepad', 'Juegos', Gamepad2),
	lu('cloud', 'Nube', Cloud),
	lu('server', 'Servidor', Server),
	lu('database', 'Datos', Database),
	lu('globe', 'Dominio/Web', Globe),
	lu('wifi', 'Internet', Wifi),
	lu('smartphone', 'Móvil', Smartphone),
	lu('code', 'Dev', Code),
	lu('palette', 'Diseño', Palette),
	lu('camera', 'Foto/Vídeo', Camera),
	lu('book', 'Lectura', BookOpen),
	lu('news', 'Noticias', Newspaper),
	lu('graduation', 'Educación', GraduationCap),
	lu('dumbbell', 'Gimnasio', Dumbbell),
	lu('heart', 'Salud', Heart),
	lu('coffee', 'Comida/Café', Coffee),
	lu('shopping', 'Compras', ShoppingBag),
	lu('car', 'Transporte', Car),
	lu('house', 'Hogar', House),
	lu('briefcase', 'Trabajo', Briefcase),
	lu('mail', 'Correo', Mail),
	lu('shield', 'Seguridad', Shield),
	lu('zap', 'Energía', Zap),
	lu('card', 'Pago', CreditCard)
];

const LU_BY_ID = new Map(LUCIDE_ICONS.map((i) => [i.id, i]));

// Resolves an id to its IconDef. 'lu:*' is static; 'si:*' comes from the lazy
// catalog (reactive: re-resolves when loading finishes).
export function iconById(id: string | null | undefined): IconDef | null {
	if (!id) return null;
	if (id.startsWith('lu:')) return LU_BY_ID.get(id) ?? null;
	return brandById(id);
}

// Popular slugs for the default grid (without dumping all 3000 at once).
const POPULAR = new Set(
	[
		'netflix',
		'spotify',
		'youtube',
		'youtubemusic',
		'primevideo',
		'max',
		'hbo',
		'appletv',
		'applemusic',
		'icloud',
		'disneyplus',
		'hulu',
		'crunchyroll',
		'twitch',
		'playstation',
		'steam',
		'github',
		'gitlab',
		'notion',
		'figma',
		'canva',
		'dropbox',
		'googledrive',
		'googlephotos',
		'proton',
		'protonmail',
		'slack',
		'discord',
		'zoom',
		'cloudflare',
		'vercel',
		'netlify',
		'digitalocean',
		'namecheap',
		'godaddy'
	].map((s) => `si:${s}`)
);

// Picker's default grid: popular brands (if already loaded) + generics.
export function defaultIcons(): IconDef[] {
	const pop = allBrands().filter((b) => POPULAR.has(b.id));
	return [...pop, ...LUCIDE_ICONS];
}

// Search: across the WHOLE brand catalog + generics (capped).
export function searchIcons(query: string, limit = 80): IconDef[] {
	const q = query.trim().toLowerCase();
	if (!q) return defaultIcons();
	const brands = allBrands().filter((b) => b.label.toLowerCase().includes(q));
	const lucide = LUCIDE_ICONS.filter((i) => i.label.toLowerCase().includes(q));
	return [...lucide, ...brands].slice(0, limit);
}

export { brandsReady };

// Resolves what to render for a sub: explicit icon → brand by name → generic.
export function resolveSubVisual(
	sub: { name: string; icon?: string | null; color?: string | null },
	catColor?: string | null
): { def: IconDef | null; brand: Brand | null; color: string } {
	const def = iconById(sub.icon);
	if (def) {
		const color = sub.color || (def.kind === 'si' ? def.color : catColor || 'var(--brand)');
		return { def, brand: null, color };
	}
	const brand = brandFor(sub.name);
	if (brand) return { def: null, brand, color: sub.color || brand.color };
	return { def: null, brand: null, color: sub.color || catColor || 'var(--brand)' };
}

// DUEO HUES palette: the logo ramp (orange→pink→purple→blue) at 88% 62%.
export const DUEO_COLORS = [
	'hsl(35 88% 62%)',
	'hsl(14 88% 62%)',
	'hsl(350 88% 62%)',
	'hsl(330 88% 62%)',
	'hsl(305 88% 62%)',
	'hsl(280 88% 62%)',
	'hsl(255 88% 62%)',
	'hsl(230 88% 62%)'
];
