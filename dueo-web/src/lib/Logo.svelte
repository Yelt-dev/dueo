<script lang="ts">
	// Wordmark "Due" + la "o" como anillo multicolor CERRADO, del tamaño de la
	// mayúscula (cap height), alineado a su base, con el bead a la derecha.
	import RingMark from './RingMark.svelte';
	let { size = 40 }: { size?: number } = $props();

	const ring = $derived(size * 0.727); // diámetro visible ≈ alto de la mayúscula (cap height)
	const stroke = $derived(size * 0.15); // ≈ grosor de los trazos de "Due" (Inter 800)

	// El SVG es más grande que el anillo (sitio para el bead). Compensamos para:
	// (1) asentar el anillo en la base (+ overshoot) y (2) respetar el espacio
	// entre la "e" y la "o".
	const dotR = $derived(stroke * 0.78);
	const pad = $derived(Math.max(0, dotR - stroke / 2) + 0.5); // margen interno del SVG
	const nudge = $derived(pad + ring * 0.02); // baja a la base + overshoot
	const gapToText = $derived(size * 0.065); // espacio visible "e" → "o"
	const ml = $derived(gapToText - pad);
</script>

<span
	class="logo"
	style="font-size:{size}px; --ring-nudge:{nudge}px; --ring-ml:{ml}px"
	role="img"
	aria-label="Dueo"
	><span class="w" aria-hidden="true">Due</span><RingMark
		size={ring}
		{stroke}
		gap={0}
		rot={40}
	/></span
>

<style>
	.logo {
		font-weight: 800;
		line-height: 1;
		color: var(--text);
		white-space: nowrap;
		user-select: none;
	}
	.w {
		letter-spacing: -0.03em;
	}
	.logo :global(.ring) {
		margin-left: var(--ring-ml);
		transform: translateY(var(--ring-nudge));
	}
</style>
