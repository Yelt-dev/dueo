// Iconos para el form: Lucide (genéricos, estáticos y ligeros) + Simple Icons
// (marcas, cargadas PEREZOSAMENTE desde brandcat para no inflar el bundle).
// El color sale de la paleta de TONOS DUEO (rampa del logo).

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

// Genéricos (Lucide) por tipo de gasto.
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

// Resuelve un id a su IconDef. 'lu:*' es estático; 'si:*' viene del catálogo
// perezoso (reactivo: re-resuelve cuando termina de cargar).
export function iconById(id: string | null | undefined): IconDef | null {
	if (!id) return null;
	if (id.startsWith('lu:')) return LU_BY_ID.get(id) ?? null;
	return brandById(id);
}

// Slugs populares para el grid por defecto (sin volcar 3000 de golpe).
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

// Grid por defecto del picker: marcas populares (si ya cargaron) + genéricos.
export function defaultIcons(): IconDef[] {
	const pop = allBrands().filter((b) => POPULAR.has(b.id));
	return [...pop, ...LUCIDE_ICONS];
}

// Búsqueda: en TODO el catálogo de marcas + genéricos (con tope).
export function searchIcons(query: string, limit = 80): IconDef[] {
	const q = query.trim().toLowerCase();
	if (!q) return defaultIcons();
	const brands = allBrands().filter((b) => b.label.toLowerCase().includes(q));
	const lucide = LUCIDE_ICONS.filter((i) => i.label.toLowerCase().includes(q));
	return [...lucide, ...brands].slice(0, limit);
}

export { brandsReady };

// Resuelve qué pintar para una sub: icono explícito → marca por nombre → genérico.
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

// Paleta de TONOS DUEO: la rampa del logo (naranja→rosa→púrpura→azul) a 88% 62%.
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
