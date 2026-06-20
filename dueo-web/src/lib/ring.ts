// Genera el anillo multicolor de Dueo como SEGMENTOS de arco (fake conic gradient).
// Cada segmento toma un tono de una rampa naranja→rosa→púrpura→azul; con
// stroke-linecap redondo y segmentos contiguos, se funden en un anillo suave.

export type Seg = { d: string; color: string };

function polar(cx: number, cy: number, r: number, deg: number): [number, number] {
	const a = ((deg - 90) * Math.PI) / 180; // 0° = arriba, sentido horario
	return [cx + r * Math.cos(a), cy + r * Math.sin(a)];
}

export type RingOpts = {
	cx: number;
	cy: number;
	r: number;
	gap?: number; // grados de hueco
	rot?: number; // rotación del conjunto (mueve hueco + bead)
	count?: number; // nº de segmentos
	sat?: number;
	light?: number;
	hueStart?: number;
	hueSpan?: number;
};

// Punto "cabeza" del spinner (extremo superior derecho, en el tono cálido inicial).
export function ringHead(o: RingOpts) {
	const { cx, cy, r, gap = 62, rot = 0, sat = 88, light = 62, hueStart = 35 } = o;
	const [x, y] = polar(cx, cy, r, gap / 2 + rot);
	return { x, y, color: `hsl(${hueStart} ${sat}% ${light}%)` };
}

export function ringSegments(o: RingOpts): Seg[] {
	const {
		cx,
		cy,
		r,
		gap = 62,
		rot = 0,
		count = 16,
		sat = 88,
		light = 62,
		hueStart = 35,
		hueSpan = 165
	} = o;
	const total = 360 - gap;
	const start = gap / 2 + rot; // inicio del arco (con rotación)
	const step = total / count;
	const segs: Seg[] = [];
	for (let i = 0; i < count; i++) {
		const a0 = start + i * step;
		const a1 = a0 + step;
		const [x0, y0] = polar(cx, cy, r, a0);
		const [x1, y1] = polar(cx, cy, r, a1);
		const t = i / (count - 1);
		const hue = (((hueStart - hueSpan * t) % 360) + 360) % 360;
		segs.push({
			d: `M ${x0.toFixed(2)} ${y0.toFixed(2)} A ${r} ${r} 0 0 1 ${x1.toFixed(2)} ${y1.toFixed(2)}`,
			color: `hsl(${hue.toFixed(0)} ${sat}% ${light}%)`
		});
	}
	return segs;
}
