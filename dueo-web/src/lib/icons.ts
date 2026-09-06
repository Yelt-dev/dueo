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
	| { id: string; kind: 'lu'; label: string; comp: Component; keywords?: string[] }
	| { id: string; kind: 'si'; label: string; path: string; color: string };

const lu = (id: string, label: string, comp: Component, keywords: string[] = []): IconDef => ({
	id: `lu:${id}`,
	kind: 'lu',
	label,
	comp,
	keywords
});

// Generic icons (Lucide) by expense type. `keywords` exist because Simple Icons
// does NOT carry every brand (Disney+, AWS, Microsoft, Adobe, HostGator… were
// removed from the package), so searching a missing brand has to land on a
// sensible generic instead of returning nothing. Both languages, unaccented.
export const LUCIDE_ICONS: IconDef[] = [
	lu('tv', 'Streaming', Tv, [
		'streaming',
		'tv',
		'disney',
		'disney+',
		'disneyplus',
		'star+',
		'hbo',
		'max',
		'prime',
		'prime video',
		'amazon prime',
		'hulu',
		'paramount',
		'peacock',
		'movistar',
		'series',
		'shows'
	]),
	lu('film', 'Cine', Film, ['cine', 'cinema', 'peliculas', 'movies', 'film', 'mubi', 'filmin']),
	lu('music', 'Música', Music, ['musica', 'music', 'spotify', 'tidal', 'deezer', 'apple music']),
	lu('headphones', 'Audio', Headphones, ['audio', 'podcast', 'audible', 'audiolibro', 'audiobook']),
	lu('gamepad', 'Juegos', Gamepad2, [
		'juegos',
		'games',
		'gaming',
		'xbox',
		'game pass',
		'playstation',
		'nintendo',
		'steam'
	]),
	lu('cloud', 'Nube', Cloud, [
		'nube',
		'cloud',
		'aws',
		'amazon',
		'amazon web services',
		's3',
		'azure',
		'microsoft',
		'gcp',
		'google cloud',
		'backup',
		'respaldo',
		'drive',
		'onedrive',
		'icloud',
		'mega',
		'storage'
	]),
	lu('server', 'Servidor', Server, [
		'servidor',
		'server',
		'vps',
		'hosting',
		'hostgator',
		'bluehost',
		'siteground',
		'hostinger',
		'linode',
		'hetzner',
		'ovh',
		'contabo',
		'dedicado',
		'dedicated',
		'cpanel',
		'plesk'
	]),
	lu('database', 'Datos', Database, [
		'datos',
		'database',
		'base de datos',
		'db',
		'sql',
		'mongo',
		'redis',
		'supabase',
		'firebase'
	]),
	lu('globe', 'Dominio/Web', Globe, [
		'dominio',
		'domain',
		'web',
		'sitio',
		'website',
		'dns',
		'ssl',
		'godaddy',
		'namecheap',
		'porkbun',
		'renovacion de dominio'
	]),
	lu('wifi', 'Internet', Wifi, ['internet', 'wifi', 'fibra', 'fiber', 'isp', 'banda ancha']),
	lu('smartphone', 'Móvil', Smartphone, [
		'movil',
		'mobile',
		'celular',
		'telefono',
		'phone',
		'plan',
		'linea',
		'prepago',
		'roaming'
	]),
	lu('code', 'Dev', Code, [
		'dev',
		'code',
		'codigo',
		'github',
		'gitlab',
		'copilot',
		'jetbrains',
		'ide',
		'api',
		'saas'
	]),
	lu('palette', 'Diseño', Palette, [
		'diseno',
		'design',
		'adobe',
		'photoshop',
		'illustrator',
		'creative cloud',
		'figma',
		'canva'
	]),
	lu('camera', 'Foto/Vídeo', Camera, ['foto', 'photo', 'video', 'camara', 'camera', 'lightroom']),
	lu('book', 'Lectura', BookOpen, ['lectura', 'libros', 'books', 'kindle', 'ebook', 'revista']),
	lu('news', 'Noticias', Newspaper, ['noticias', 'news', 'periodico', 'prensa', 'diario']),
	lu('graduation', 'Educación', GraduationCap, [
		'educacion',
		'education',
		'curso',
		'course',
		'udemy',
		'coursera',
		'platzi',
		'escuela'
	]),
	lu('dumbbell', 'Gimnasio', Dumbbell, ['gimnasio', 'gym', 'fitness', 'deporte', 'entrenamiento']),
	lu('heart', 'Salud', Heart, [
		'salud',
		'health',
		'medico',
		'seguro medico',
		'terapia',
		'dentista'
	]),
	lu('coffee', 'Comida/Café', Coffee, [
		'comida',
		'food',
		'cafe',
		'coffee',
		'restaurante',
		'delivery'
	]),
	lu('shopping', 'Compras', ShoppingBag, ['compras', 'shopping', 'tienda', 'store', 'membresia']),
	lu('car', 'Transporte', Car, [
		'transporte',
		'transport',
		'coche',
		'auto',
		'car',
		'uber',
		'peaje'
	]),
	lu('house', 'Hogar', House, [
		'hogar',
		'home',
		'casa',
		'renta',
		'alquiler',
		'rent',
		'luz',
		'agua'
	]),
	lu('briefcase', 'Trabajo', Briefcase, ['trabajo', 'work', 'oficina', 'office', 'negocio']),
	lu('mail', 'Correo', Mail, ['correo', 'mail', 'email', 'buzon', 'newsletter', 'proton']),
	lu('shield', 'Seguridad', Shield, [
		'seguridad',
		'security',
		'vpn',
		'antivirus',
		'password',
		'contrasenas',
		'1password',
		'bitwarden',
		'seguro',
		'insurance'
	]),
	lu('zap', 'Energía', Zap, ['energia', 'energy', 'luz', 'electricidad', 'gas', 'utilities']),
	lu('card', 'Pago', CreditCard, ['pago', 'payment', 'banco', 'bank', 'comision', 'tarjeta'])
];

const LU_BY_ID = new Map(LUCIDE_ICONS.map((i) => [i.id, i]));

// Lowercase and strip accents, so "Música"/"musica" and "Diseño"/"diseno" match.
function norm(s: string): string {
	return s
		.toLowerCase()
		.normalize('NFD')
		.replace(/[\u0300-\u036f]/g, '');
}

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
		'max',
		'hbo',
		'appletv',
		'applemusic',
		'icloud',
		'crunchyroll',
		'twitch',
		'playstation',
		'steam',
		'github',
		'gitlab',
		'notion',
		'figma',
		'dropbox',
		'googledrive',
		'googlephotos',
		'proton',
		'protonmail',
		'slack',
		'discord',
		'zoom',
		'openai',
		'anthropic',
		'claude',
		'cloudflare',
		'vercel',
		'netlify',
		'digitalocean',
		'namecheap',
		'godaddy',
		'hetzner',
		'ovh',
		'ionos',
		'vultr',
		'porkbun'
	].map((s) => `si:${s}`)
);

// Picker's default grid: popular brands (if already loaded) + generics.
export function defaultIcons(): IconDef[] {
	const pop = allBrands().filter((b) => POPULAR.has(b.id));
	return [...pop, ...LUCIDE_ICONS];
}

// Search: across the WHOLE brand catalog + generics (capped).
export function searchIcons(query: string, limit = 80): IconDef[] {
	const q = norm(query);
	if (!q) return defaultIcons();
	// Brands match on title AND slug, so "hbomax" or "youtubemusic" work too.
	const brands = allBrands().filter((b) => norm(b.label).includes(q) || b.id.slice(3).includes(q));
	// Generics match on label or keyword, which is what catches the brands the
	// catalog doesn't ship: "aws" → Nube, "hostgator" → Servidor, "disney" → Streaming.
	const lucide = LUCIDE_ICONS.filter(
		(i) =>
			norm(i.label).includes(q) ||
			(i.kind === 'lu' && i.keywords?.some((k) => k.includes(q) || q.includes(k)))
	);
	return [...brands, ...lucide].slice(0, limit);
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
