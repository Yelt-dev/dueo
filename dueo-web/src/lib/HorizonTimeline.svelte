<script lang="ts">
	import { tick } from 'svelte';
	import { Receipt, CreditCard, Wallet, Flag } from '@lucide/svelte';
	import { timeColor } from './format';
	import { i18n, locale } from './i18n.svelte';
	import Icon from './Icon.svelte';
	import { resolveSubVisual } from './icons';

	// `kind` is what actually happens on that date, which is the distinction that
	// matters when scanning the line: 'manual' needs you to do something, 'auto'
	// is just money leaving, 'end' is the contract finishing.
	type Kind = 'manual' | 'auto' | 'end';
	type Item = {
		id: number;
		name: string;
		days: number;
		progress: number;
		kind?: Kind;
		icon?: string | null;
		color?: string | null;
	};
	const KIND_ICON = { manual: Wallet, auto: CreditCard, end: Flag };
	const KIND_LABEL: Record<Kind, string> = {
		manual: 'hz.kindManual',
		auto: 'hz.kindAuto',
		end: 'hz.kindEnd'
	};
	let { items = [], onselect }: { items?: Item[]; onselect?: (id: number) => void } = $props();

	// --- geometry ---
	const LINE_Y = 52; // line y (from bottom); leaves room for ticks
	const BASE_STEM = 32;
	const LANE = 48; // VERTICAL spacing between branches
	const BRANCH_X = 38; // HORIZONTAL offset per branch (git/tree effect)
	const SLOT = 112; // width reserved per marker (visible name) → anti-overlap
	const LEFT_PAD = 64;

	let viewportEl: HTMLDivElement;
	let trackW = $state(0);
	let pxPerDay = $state(0);
	let activePreset = $state(readPreset());

	function readPreset(): string {
		try {
			// '1A' was removed → if persisted from before, fall back to 'Todo'.
			const saved = localStorage.getItem('dueo_horizon_preset');
			return saved && saved !== '1A' ? saved : 'Todo';
		} catch {
			return 'Todo';
		}
	}

	const PRESETS = [
		{ label: '1M', days: 30 },
		{ label: '3M', days: 91 },
		{ label: '6M', days: 182 },
		{ label: 'Todo', days: null as number | null }
	];

	const maxDays = $derived(Math.max(30, ...items.map((i) => i.days)));

	function ppdForPreset(label: string): number {
		const p = PRESETS.find((x) => x.label === label) ?? PRESETS[3];
		const d = p.days ?? maxDays + 15;
		return clampPPD(trackW / Math.max(d, 7));
	}

	// Apply the saved preset once the width is known.
	$effect(() => {
		if (trackW > 0 && pxPerDay === 0) pxPerDay = ppdForPreset(activePreset);
	});

	// PROXIMITY opacity: the soonest-due item is fully prominent and fades toward a
	// minimum as it moves further into the future. A base dim, softer than the hover
	// one, so the view prioritizes upcoming items without hiding distant ones.
	function proximityOpacity(days: number): number {
		const d = Math.max(0, days);
		if (d <= 7) return 1; // next week = max prominence
		const t = Math.min(1, (d - 7) / (180 - 7)); // 7 to 180 days
		return 1 - 0.55 * t; // 1.0 → 0.45
	}

	// Item under the cursor: focusing one brings its branch/node to full opacity and
	// dims the rest (matches the chip dim). null = nothing focused → base dim.
	let hoveredId = $state<number | null>(null);
	function branchOp(m: { id: number; op: number }): number {
		if (hoveredId === null) return m.op * 0.85;
		return m.id === hoveredId ? 1 : 0.15;
	}
	function nodeOp(m: { id: number; op: number }): number {
		if (hoveredId === null) return m.op;
		return m.id === hoveredId ? 1 : 0.15;
	}

	// --- tree-like branch layout (anti-overlap) ---
	// The NODE (dot) stays at its real date on the line (x); the CHIP branches off
	// diagonally: each collision moves it up a lane (by) and right (bx), joined by a
	// curved connector. So nothing ends up "directly behind".
	const laidOut = $derived.by(() => {
		const sorted = [...items].sort((a, b) => a.days - b.days);
		const laneLast: number[] = [];
		return sorted.map((it) => {
			const x = Math.max(0, it.days) * pxPerDay + LEFT_PAD;
			let lane = 0;
			while (lane < laneLast.length && x - laneLast[lane] < SLOT) lane++;
			laneLast[lane] = x;
			const vis = resolveSubVisual(it);
			return {
				...it,
				x, // node on the line (real date)
				lane,
				by: BASE_STEM + lane * LANE, // branch height
				bx: x + lane * BRANCH_X, // chip x (shifted per branch)
				color: timeColor(it.progress), // URGENCY color (line/node/days)
				def: vis.def, // explicit or brand icon
				brand: vis.brand,
				chipColor: vis.color, // CHIP color (brand/chosen)
				kind: (it.kind ?? 'manual') as Kind,
				// Only actionable dates pulse: an automatic charge can't be missed.
				urgent: it.days <= 7 && (it.kind ?? 'manual') !== 'auto',
				op: proximityOpacity(it.days) // base dim by distance
			};
		});
	});
	const maxLane = $derived(laidOut.length ? Math.max(...laidOut.map((m) => m.lane)) : 0);
	const trackHeight = $derived(LINE_Y + 152 + (BASE_STEM + maxLane * LANE));
	const lineY = $derived(trackHeight - LINE_Y); // line y from the TOP (for the SVG)

	// Curved connector from node (x, lineY) to chip (bx, lineY - by): a cubic that
	// exits vertical and enters vertical → smooth git-branch-style curve.
	function branchPath(m: { x: number; bx: number; by: number }, ly: number): string {
		const x0 = m.x,
			y0 = ly;
		const x1 = m.bx,
			y1 = ly - m.by;
		const my = (y0 + y1) / 2;
		return `M ${x0} ${y0} C ${x0} ${my}, ${x1} ${my}, ${x1} ${y1}`;
	}

	// tick interval based on zoom (weeks → months → quarters → years)
	const tickStep = $derived.by(() => {
		if (pxPerDay <= 0) return 365;
		const windowDays = trackW / pxPerDay;
		return windowDays <= 45 ? 7 : windowDays <= 130 ? 30 : windowDays <= 400 ? 91 : 365;
	});
	// track extends to the last tick → fills the period (no gap at the end)
	const endDays = $derived(Math.ceil((maxDays + 1) / tickStep) * tickStep);
	const trackWidth = $derived(
		Math.max(trackW, endDays * pxPerDay + LEFT_PAD + 24 + maxLane * BRANCH_X)
	);

	const ticks = $derived.by(() => {
		if (pxPerDay <= 0) return [] as { x: number; label: string }[];
		const today = new Date();
		const out: { x: number; label: string }[] = [];
		for (let d = 0; d <= endDays; d += tickStep) {
			const date = new Date(today.getTime() + d * 86_400_000);
			let label: string;
			const loc = locale();
			if (tickStep === 7) label = date.toLocaleDateString(loc, { day: '2-digit', month: 'short' });
			else if (tickStep <= 31) label = date.toLocaleDateString(loc, { month: 'short' });
			else if (tickStep <= 92)
				label =
					date.toLocaleDateString(loc, { month: 'short' }) +
					" '" +
					String(date.getFullYear()).slice(2);
			else label = String(date.getFullYear());
			out.push({ x: d * pxPerDay + LEFT_PAD, label });
		}
		return out;
	});

	// Line gradient by urgency: red (near NOW) → amber → green.
	const lineGrad = $derived.by(() => {
		if (pxPerDay <= 0) return 'var(--border)';
		const xR = LEFT_PAD + 14 * pxPerDay; // up to ~14 days = red
		const xA = LEFT_PAD + 45 * pxPerDay; // ~45 days = amber
		const xG = LEFT_PAD + 85 * pxPerDay; // from ~85 days = green
		return `linear-gradient(90deg,
			var(--danger) 0px,
			var(--danger) ${xR.toFixed(0)}px,
			var(--warn) ${xA.toFixed(0)}px,
			var(--ok) ${xG.toFixed(0)}px,
			var(--ok) 100%)`;
	});

	// --- zoom / pan ---
	function clampPPD(v: number) {
		const min = trackW / Math.max(maxDays + 30, 60);
		const max = trackW / 7;
		return Math.min(max, Math.max(min, v));
	}

	function setPreset(p: (typeof PRESETS)[number]) {
		activePreset = p.label;
		try {
			localStorage.setItem('dueo_horizon_preset', p.label);
		} catch {
			// localStorage unavailable (private mode): ignored.
		}
		pxPerDay = ppdForPreset(p.label);
		requestAnimationFrame(() => viewportEl && (viewportEl.scrollLeft = 0));
	}

	// While zooming, the markers must NOT glide to their new x: the branches and
	// nodes are SVG (no transition) and would leave the chip trailing behind for
	// 0.4s. Cleared shortly after the last wheel event so hovering keeps its ease.
	let zooming = $state(false);
	let zoomTimer: ReturnType<typeof setTimeout>;

	async function onWheel(e: WheelEvent) {
		// Zoom ONLY with trackpad pinch (sets ctrlKey) or Ctrl/Cmd + wheel.
		// Without modifier: plain wheel = page scroll; lateral = native pan.
		if (!(e.ctrlKey || e.metaKey)) return;
		e.preventDefault();
		// Zoom proportional to how much was actually scrolled, not a fixed step per
		// event: a trackpad pinch fires many tiny deltas (smooth) and a mouse notch
		// one big one (~6%). deltaMode normalizes browsers that report lines/pages.
		const unit = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? 400 : 1;
		const factor = Math.min(1.25, Math.max(0.8, Math.exp(-e.deltaY * unit * 0.0006)));
		const newPPD = clampPPD(pxPerDay * factor);
		if (newPPD === pxPerDay) return; // hit clamp → no change, keeps preset
		const rect = viewportEl.getBoundingClientRect();
		const cx = e.clientX - rect.left;
		const dayAtCursor = (viewportEl.scrollLeft + cx - LEFT_PAD) / pxPerDay;
		zooming = true;
		clearTimeout(zoomTimer);
		zoomTimer = setTimeout(() => (zooming = false), 140);
		pxPerDay = newPPD;
		activePreset = ''; // a real zoom happened now
		// `tick()` (not rAF): waits for the track's new width to be in the DOM and
		// corrects the scroll BEFORE the browser paints, so there is no in-between
		// frame where the content sits at the new scale with the old scroll.
		await tick();
		viewportEl.scrollLeft = dayAtCursor * pxPerDay + LEFT_PAD - cx;
	}

	let dragging = $state(false);
	let dragX = 0;
	let dragScroll = 0;
	function onPointerDown(e: PointerEvent) {
		dragging = true;
		dragX = e.clientX;
		dragScroll = viewportEl.scrollLeft;
		viewportEl.setPointerCapture?.(e.pointerId);
	}
	function onPointerMove(e: PointerEvent) {
		if (dragging) viewportEl.scrollLeft = dragScroll - (e.clientX - dragX);
	}
	function onPointerUp() {
		dragging = false;
	}
</script>

<div class="horizon acrylic">
	<header>
		<div class="title">
			<span>{i18n.t('hz.title')}</span>
			<span class="sub">{i18n.t('hz.hint')}</span>
		</div>
		<ul class="legend">
			{#each ['manual', 'auto', 'end'] as const as k (k)}
				{@const KIcon = KIND_ICON[k]}
				<li>
					<KIcon size={14} />
					{i18n.t(KIND_LABEL[k])}
				</li>
			{/each}
		</ul>
		<div class="presets">
			{#each PRESETS as p (p.label)}
				<button class:active={activePreset === p.label} onclick={() => setPreset(p)}
					>{p.label === 'Todo' ? i18n.t('hz.all') : p.label}</button
				>
			{/each}
		</div>
	</header>

	<!-- pan/zoom is progressive enhancement (mouse); the accessible content is the
	     markers, which are keyboard-navigable buttons. -->
	<div
		class="viewport"
		class:grabbing={dragging}
		role="group"
		aria-label={i18n.t('hz.title')}
		bind:this={viewportEl}
		bind:clientWidth={trackW}
		onwheel={onWheel}
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointerleave={onPointerUp}
	>
		<div class="track" class:zooming style="width:{trackWidth}px; height:{trackHeight}px">
			{#if pxPerDay > 0}
				{#each ticks as t, i (i)}
					<div class="tick" style="left:{t.x}px">
						<span class="grid" style="height:{trackHeight - LINE_Y}px"></span>
						<span class="tickdot" style="bottom:{LINE_Y - 6}px"></span>
						{#if i > 0}<span class="tlabel">{t.label}</span>{/if}
					</div>
				{/each}

				<div class="line-glow" style="bottom:{LINE_Y - 6}px; background:{lineGrad}"></div>
				<div class="line" style="bottom:{LINE_Y - 3}px; background:{lineGrad}"></div>
				<div class="hoy" style="bottom:{LINE_Y}px; left:{LEFT_PAD}px">
					<span>{i18n.t('hz.now')}</span>
				</div>

				<!-- BRANCH layer: curved connectors + nodes on the line -->
				<svg class="branches" width={trackWidth} height={trackHeight} aria-hidden="true">
					{#each laidOut as m (m.id)}
						<path
							class="branch"
							class:urgent={m.urgent}
							class:auto={m.kind === 'auto'}
							d={branchPath(m, lineY)}
							style="stroke:{m.color}; opacity:{branchOp(m)}"
						/>
						{#if m.kind === 'end'}
							<rect
								class="node"
								x={m.x - 6}
								y={lineY - 6}
								width="12"
								height="12"
								transform="rotate(45 {m.x} {lineY})"
								style="fill:{m.color}; opacity:{nodeOp(m)}"
							/>
						{:else if m.kind === 'auto'}
							<circle
								class="node hollow"
								cx={m.x}
								cy={lineY}
								r="6"
								style="stroke:{m.color}; opacity:{nodeOp(m)}"
							/>
						{:else}
							<circle
								class="node"
								class:urgent={m.urgent}
								cx={m.x}
								cy={lineY}
								r="7"
								style="fill:{m.color}; opacity:{nodeOp(m)}"
							/>
						{/if}
					{/each}
				</svg>

				{#each laidOut as m (m.id)}
					{@const KIcon = KIND_ICON[m.kind]}
					<div
						class="marker"
						class:urgent={m.urgent}
						role="button"
						tabindex="0"
						title={i18n.t('hz.viewInList')}
						onmouseenter={() => (hoveredId = m.id)}
						onmouseleave={() => (hoveredId = null)}
						onpointerdown={(e) => e.stopPropagation()}
						onclick={() => onselect?.(m.id)}
						onkeydown={(e) =>
							(e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), onselect?.(m.id))}
						style="bottom:{LINE_Y +
							m.by}px; --mx:{m.bx}px; --op:{m.op}; --c:{m.color}; --bc:{m.chipColor}"
					>
						<span class="name" title={m.name}>{m.name}</span>
						<span class="days">
							<KIcon size={13} />
							{m.days}d
						</span>
						<span class="chip">
							{#if m.def}
								<Icon def={m.def} size={23} />
							{:else if m.brand}
								<svg viewBox="0 0 24 24" width="23" height="23" fill="currentColor"
									><path d={m.brand.path} /></svg
								>
							{:else}
								<Receipt size={23} />
							{/if}
						</span>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</div>

<style>
	.horizon {
		border-radius: var(--radius-xl, 20px);
		padding: 1.1rem 0 0;
		margin-bottom: var(--gap-section);
		overflow: hidden;
	}
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		padding: 0 1.4rem 0.9rem;
	}
	.title {
		display: flex;
		flex-direction: column;
	}
	.title span:first-child {
		font-weight: 700;
		font-size: 0.95rem;
	}
	.sub {
		color: var(--text-muted);
		font-size: 0.74rem;
	}
	/* Legend: the node shapes carry meaning, so they need naming once. */
	.legend {
		display: flex;
		flex-wrap: wrap;
		gap: 0.25rem 1rem;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: 0.79rem;
		color: var(--text-2);
	}
	.legend li {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.presets {
		display: flex;
		gap: 2px;
		padding: 3px;
		border-radius: 10px;
		background: var(--surface-2);
		border: 1px solid var(--border);
	}
	.presets button {
		border: none;
		background: transparent;
		color: var(--text-2);
		font-size: 0.78rem;
		font-weight: 600;
		padding: 4px 9px;
		border-radius: 7px;
		cursor: pointer;
		transition:
			color 0.12s,
			background 0.12s;
	}
	.presets button:hover {
		color: var(--text);
	}
	.presets button.active {
		color: white;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
	}

	/* Mobile: title+subtitle and presets no longer fit on one row → presets
	   drop to their own full-width row below the header. */
	@media (max-width: 640px) {
		header {
			flex-direction: column;
			align-items: stretch;
			gap: 0.7rem;
			padding: 0 1rem 0.8rem;
		}
		.legend {
			order: 3;
		}
		.presets {
			align-self: stretch;
		}
		.presets button {
			flex: 1;
			text-align: center;
		}
	}

	.viewport {
		overflow-x: auto;
		overflow-y: hidden;
		cursor: grab;
		/* hidden scrollbar */
		scrollbar-width: none;
		-ms-overflow-style: none;
	}
	.viewport::-webkit-scrollbar {
		display: none;
	}
	.viewport.grabbing {
		cursor: grabbing;
	}
	.track {
		position: relative;
		min-width: 100%;
	}

	.line-glow {
		position: absolute;
		left: 0;
		right: 0;
		height: 11px;
		border-radius: 11px;
		filter: blur(9px);
		opacity: var(--horizon-glow, 0.5);
		pointer-events: none;
	}
	.line {
		position: absolute;
		left: 0;
		right: 0;
		height: 6px;
		border-radius: 6px;
	}
	.hoy {
		position: absolute;
		height: 22px;
	}
	.hoy span {
		position: absolute;
		bottom: 7px;
		left: 4px;
		font-size: 0.7rem;
		font-weight: 800;
		letter-spacing: 0.1em;
		color: var(--text-muted);
	}

	.tick {
		position: absolute;
		bottom: 0;
		transform: translateX(-50%);
	}
	.grid {
		position: absolute;
		bottom: 30px;
		left: 0;
		width: 1px;
		background: linear-gradient(var(--border) 0, transparent 55%);
	}
	.tickdot {
		position: absolute;
		left: 50%;
		transform: translateX(-50%);
		width: 12px;
		height: 12px;
		border-radius: 50%;
		background: var(--text-muted);
		border: 2px solid var(--surface);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--text-muted) 16%, transparent);
		z-index: 1;
	}
	.tlabel {
		position: absolute;
		bottom: 8px;
		left: 50%;
		transform: translateX(-50%);
		font-size: 0.72rem;
		color: var(--text-muted);
		white-space: nowrap;
	}

	/* SVG branch layer: behind the chips, does not intercept the mouse. */
	.branches {
		position: absolute;
		top: 0;
		left: 0;
		z-index: 1;
		pointer-events: none;
		overflow: visible;
	}
	.branch {
		fill: none;
		stroke-width: 3;
		opacity: 0.85;
	}
	.node {
		stroke: var(--surface);
		stroke-width: 2.5;
	}
	.node.hollow {
		fill: none;
		stroke-width: 2;
	}
	/* An automatic charge is information, not a task: dashed and calm. */
	.branch.auto {
		stroke-dasharray: 3 4;
	}
	.node.urgent {
		animation: nodepulse 1.6s ease-in-out infinite;
	}
	@keyframes nodepulse {
		0%,
		100% {
			r: 6px;
		}
		50% {
			r: 8.5px;
		}
	}

	.marker {
		position: absolute;
		left: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		transform: translateX(var(--mx)) translateX(-50%);
		transform-origin: bottom center;
		z-index: 2;
		cursor: pointer;
		/* base dim by distance (proximityOpacity); hover-dim overrides it */
		opacity: var(--op, 1);
		transition:
			transform 0.4s cubic-bezier(0.2, 0.8, 0.2, 1),
			opacity 0.16s ease;
		will-change: transform;
	}
	/* Zoom repositions everything at once: chips move with their branch, not after it. */
	.track.zooming .marker {
		transition: none;
	}
	.marker:hover {
		transform: translateX(var(--mx)) translateX(-50%) scale(1.08);
		z-index: 10;
		opacity: 1; /* focused item ALWAYS full, even if distant (base dim) */
		transition: transform 0.16s ease;
	}
	/* hovering a marker dims the others to focus the active one */
	.track:has(.marker:hover) .marker:not(:hover) {
		opacity: 0.3;
	}
	/* branches/nodes transition their opacity (driven by branchOp/nodeOp) */
	.branch,
	.node {
		transition: opacity 0.16s ease;
	}
	/* full name on hover */
	.marker:hover .name {
		max-width: 190px;
	}
	.name {
		max-width: 104px;
		margin-bottom: 7px;
		font-size: 0.86rem;
		font-weight: 650;
		color: var(--text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	/* URGENCY = number of days (time color) */
	.days {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		margin-bottom: 6px;
		font-size: 0.82rem;
		font-weight: 800;
		color: var(--c);
		font-variant-numeric: tabular-nums;
	}
	/* CHIP = brand only (logo + tint of its color), the branch tip. */
	.chip {
		display: grid;
		place-items: center;
		width: 46px;
		height: 46px;
		border-radius: 50%;
		color: var(--bc);
		background: color-mix(in srgb, var(--bc) 16%, var(--surface));
		border: 1px solid color-mix(in srgb, var(--bc) 38%, transparent);
		box-shadow: 0 3px 10px -3px color-mix(in srgb, var(--bc) 50%, transparent);
		transition: transform 0.15s;
	}
</style>
