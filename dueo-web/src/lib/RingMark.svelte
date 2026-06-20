<script lang="ts">
	import { ringSegments, ringHead } from './ring';

	// size = diámetro VISIBLE del anillo (px); stroke = grosor (px);
	// gap = grados de hueco; rot = rotación (mueve hueco + bead).
	let {
		size = 28,
		stroke = 5,
		gap = 30,
		rot = 0
	}: { size?: number; stroke?: number; gap?: number; rot?: number } = $props();

	// Todo derivado de los props (reactivo): el SVG se agranda para que el bead no
	// se corte; el anillo visible = size.
	const dotR = $derived(stroke * 0.78);
	const overflow = $derived(Math.max(0, dotR - stroke / 2));
	const box = $derived(size + 2 * overflow + 1);
	const c = $derived(box / 2);
	const rMid = $derived(size / 2 - stroke / 2); // radio del centro del trazo
	const opts = $derived({ cx: c, cy: c, r: rMid, gap, rot, count: 18 });
	const segs = $derived(ringSegments(opts));
	const head = $derived(ringHead(opts));
</script>

<svg class="ring" width={box} height={box} viewBox="0 0 {box} {box}" aria-hidden="true">
	{#each segs as s, i (i)}
		<path d={s.d} fill="none" stroke={s.color} stroke-width={stroke} stroke-linecap="round" />
	{/each}
	<circle cx={head.x} cy={head.y} r={dotR} fill={head.color} />
</svg>

<style>
	.ring {
		display: inline-block;
		vertical-align: baseline;
	}
	.ring,
	.ring :global(path),
	.ring :global(circle) {
		box-shadow: none;
		filter: none;
	}
</style>
