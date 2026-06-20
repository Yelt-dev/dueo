<script lang="ts">
	import { Tween } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import { timeColor } from './format';
	import { i18n } from './i18n.svelte';

	let {
		progress = 0, // 0..1: fracción de vida ya consumida
		days = 0, // días restantes (lo que mostramos en grande)
		size = 150
	}: { progress?: number; days?: number; size?: number } = $props();

	const stroke = 12;
	const radius = $derived((size - stroke) / 2);
	const circ = $derived(2 * Math.PI * radius);

	// Animación: el anillo se "llena" suavemente al aparecer / cambiar.
	const t = new Tween(0, { duration: 900, easing: cubicOut });
	$effect(() => {
		t.target = progress;
	});

	// $derived = se recalcula solo cuando cambia algo de lo que depende.
	let offset = $derived(circ * (1 - t.current));
	// Same time-urgency ramp as the rest of the app (themeable CSS vars).
	let color = $derived(timeColor(progress));
</script>

<svg width={size} height={size} viewBox="0 0 {size} {size}">
	<!-- pista de fondo -->
	<circle
		cx={size / 2}
		cy={size / 2}
		r={radius}
		fill="none"
		stroke="var(--surface-2)"
		stroke-width={stroke}
	/>
	<!-- progreso -->
	<circle
		cx={size / 2}
		cy={size / 2}
		r={radius}
		fill="none"
		stroke={color}
		stroke-width={stroke}
		stroke-linecap="round"
		stroke-dasharray={circ}
		stroke-dashoffset={offset}
		transform="rotate(-90 {size / 2} {size / 2})"
		style="filter: drop-shadow(0 0 6px {color}88)"
	/>
	<text x="50%" y="48%" text-anchor="middle" class="num" fill={color}>{days}</text>
	<text x="50%" y="64%" text-anchor="middle" class="lbl">{i18n.t('ring.daysLeft')}</text>
</svg>

<style>
	.num {
		font-size: 2.4rem;
		font-weight: 800;
		font-variant-numeric: tabular-nums;
	}
	.lbl {
		font-size: 0.72rem;
		fill: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}
</style>
