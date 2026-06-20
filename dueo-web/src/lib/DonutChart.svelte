<script lang="ts">
	// Donut SVG sin librería. `pathLength=100` → trabajamos en porcentaje (sin
	// calcular circunferencias). Cada segmento es un arco coloreado.
	type Item = { label: string; value: number; color: string };
	let {
		items = [],
		size = 168,
		thickness = 24,
		centerTop = '',
		centerSub = ''
	}: {
		items?: Item[];
		size?: number;
		thickness?: number;
		centerTop?: string;
		centerSub?: string;
	} = $props();

	const r = $derived((size - thickness) / 2);
	const total = $derived(items.reduce((a, i) => a + i.value, 0));

	const segs = $derived.by(() => {
		const t = total || 1;
		let acc = 0;
		return items
			.filter((i) => i.value > 0)
			.map((it) => {
				const pct = (it.value / t) * 100;
				const seg = { ...it, pct, offset: -acc };
				acc += pct;
				return seg;
			});
	});
</script>

<svg width={size} height={size} viewBox="0 0 {size} {size}" role="img">
	<g transform="rotate(-90 {size / 2} {size / 2})">
		<!-- pista de fondo -->
		<circle
			cx={size / 2}
			cy={size / 2}
			{r}
			fill="none"
			stroke="color-mix(in srgb, var(--text-muted) 16%, transparent)"
			stroke-width={thickness}
		/>
		{#each segs as s (s.label)}
			<circle
				cx={size / 2}
				cy={size / 2}
				{r}
				fill="none"
				stroke={s.color}
				stroke-width={thickness}
				pathLength="100"
				stroke-dasharray="{s.pct} 100"
				stroke-dashoffset={s.offset}
				stroke-linecap="butt"
			/>
		{/each}
	</g>
	{#if centerTop}
		<text x="50%" y="48%" text-anchor="middle" class="ctop">{centerTop}</text>
	{/if}
	{#if centerSub}
		<text x="50%" y="62%" text-anchor="middle" class="csub">{centerSub}</text>
	{/if}
</svg>

<style>
	circle {
		transition: stroke-dasharray 0.5s cubic-bezier(0.2, 0.8, 0.2, 1);
	}
	.ctop {
		fill: var(--text);
		font-size: 1.05rem;
		font-weight: 750;
		font-variant-numeric: tabular-nums;
	}
	.csub {
		fill: var(--text-muted);
		font-size: 0.66rem;
	}
</style>
