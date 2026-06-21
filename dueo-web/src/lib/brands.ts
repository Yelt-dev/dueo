// Brand logos (Simple Icons). Detects the brand from the service name.
// On no match, the component falls back to the category icon (Lucide).

import {
	siNetflix,
	siSpotify,
	siYoutube,
	siApplemusic,
	siAppletv,
	siIcloud,
	siGithub,
	siNotion,
	siFigma,
	siDropbox,
	siCloudflare,
	siVercel,
	siNamecheap,
	siGodaddy,
	siDigitalocean,
	siGoogledrive,
	siGooglephotos,
	siProtonmail,
	siPlaystation,
	siHbo,
	siMax
} from 'simple-icons';

export type Brand = { path: string; color: string; label: string };

type Si = { path: string; hex: string; title: string };
const b = (s: Si): Brand => ({ path: s.path, color: '#' + s.hex, label: s.title });

// Order matters: most specific first (applemusic before any 'apple').
const MAP: { kw: string; brand: Brand }[] = [
	{ kw: 'netflix', brand: b(siNetflix) },
	{ kw: 'spotify', brand: b(siSpotify) },
	{ kw: 'youtube', brand: b(siYoutube) },
	{ kw: 'applemusic', brand: b(siApplemusic) },
	{ kw: 'appletv', brand: b(siAppletv) },
	{ kw: 'icloud', brand: b(siIcloud) },
	{ kw: 'github', brand: b(siGithub) },
	{ kw: 'notion', brand: b(siNotion) },
	{ kw: 'figma', brand: b(siFigma) },
	{ kw: 'dropbox', brand: b(siDropbox) },
	{ kw: 'cloudflare', brand: b(siCloudflare) },
	{ kw: 'vercel', brand: b(siVercel) },
	{ kw: 'namecheap', brand: b(siNamecheap) },
	{ kw: 'godaddy', brand: b(siGodaddy) },
	{ kw: 'digitalocean', brand: b(siDigitalocean) },
	{ kw: 'googledrive', brand: b(siGoogledrive) },
	{ kw: 'googlephotos', brand: b(siGooglephotos) },
	{ kw: 'proton', brand: b(siProtonmail) },
	{ kw: 'playstation', brand: b(siPlaystation) },
	{ kw: 'hbomax', brand: b(siMax) },
	{ kw: 'hbo', brand: b(siHbo) }
];

export function brandFor(name: string): Brand | null {
	const n = name.toLowerCase().replace(/[^a-z0-9]/g, '');
	for (const { kw, brand } of MAP) if (n.includes(kw)) return brand;
	return null;
}
