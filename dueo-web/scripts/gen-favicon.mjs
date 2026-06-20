// Genera src/lib/assets/favicon.svg: tile navy + anillo multicolor + cabeza del
// spinner (mismo algoritmo que src/lib/ring.ts). Correr: node scripts/gen-favicon.mjs
import { writeFileSync } from 'node:fs';

function polar(cx, cy, r, deg) {
	const a = ((deg - 90) * Math.PI) / 180;
	return [cx + r * Math.cos(a), cy + r * Math.sin(a)];
}

function segs({
	cx,
	cy,
	r,
	gap = 62,
	rot = 0,
	count = 20,
	sat = 88,
	light = 62,
	hueStart = 35,
	hueSpan = 165
}) {
	const total = 360 - gap;
	const start = gap / 2 + rot;
	const step = total / count;
	const out = [];
	for (let i = 0; i < count; i++) {
		const a0 = start + i * step;
		const a1 = a0 + step;
		const [x0, y0] = polar(cx, cy, r, a0);
		const [x1, y1] = polar(cx, cy, r, a1);
		const t = i / (count - 1);
		const hue = (((hueStart - hueSpan * t) % 360) + 360) % 360;
		out.push(
			`<path d="M ${x0.toFixed(2)} ${y0.toFixed(2)} A ${r} ${r} 0 0 1 ${x1.toFixed(2)} ${y1.toFixed(2)}" stroke="hsl(${hue.toFixed(0)} ${sat}% ${light}%)"/>`
		);
	}
	return out.join('\n    ');
}

const R = { cx: 32, cy: 32, r: 19, gap: 0, rot: 40 };
const W = 7; // grosor (consistente con el wordmark)
const paths = segs(R);
const [hx, hy] = polar(R.cx, R.cy, R.r, R.gap / 2 + R.rot); // cabeza a la derecha

const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" role="img" aria-label="Dueo">
  <defs>
    <linearGradient id="tile" x1="0" y1="0" x2="0" y2="64" gradientUnits="userSpaceOnUse">
      <stop offset="0" stop-color="#0E1428"/>
      <stop offset="1" stop-color="#070B16"/>
    </linearGradient>
    <filter id="glow" x="-30%" y="-30%" width="160%" height="160%">
      <feGaussianBlur stdDeviation="1.7"/>
    </filter>
  </defs>
  <rect width="64" height="64" rx="16" fill="url(#tile)"/>
  <g fill="none" stroke-width="${W}" stroke-linecap="round" filter="url(#glow)" opacity="0.55">
    ${paths}
  </g>
  <g fill="none" stroke-width="${W}" stroke-linecap="round">
    ${paths}
  </g>
  <circle cx="${hx.toFixed(2)}" cy="${hy.toFixed(2)}" r="${(W * 0.8).toFixed(2)}" fill="hsl(35 88% 62%)"/>
</svg>
`;

writeFileSync(new URL('../src/lib/assets/favicon.svg', import.meta.url), svg);
console.log('favicon.svg generado');
