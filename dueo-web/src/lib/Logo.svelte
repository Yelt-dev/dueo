<script lang="ts">
	// Wordmark "Due" + the "o" as a CLOSED multicolor ring, sized to the cap
	// height, baseline-aligned, with the bead on the right.
	import RingMark from './RingMark.svelte';
	let { size = 40 }: { size?: number } = $props();

	const ring = $derived(size * 0.727); // visible diameter ≈ cap height
	const stroke = $derived(size * 0.15); // ≈ stroke thickness of "Due" (Inter 800)

	// The SVG is larger than the ring (room for the bead). We compensate to:
	// (1) seat the ring on the baseline (+ overshoot) and (2) preserve the gap
	// between "e" and "o".
	const dotR = $derived(stroke * 0.78);
	const pad = $derived(Math.max(0, dotR - stroke / 2) + 0.5); // SVG inner margin
	const nudge = $derived(pad + ring * 0.02); // lower to baseline + overshoot
	const gapToText = $derived(size * 0.065); // visible "e" → "o" gap
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
