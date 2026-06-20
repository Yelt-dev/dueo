<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { fly } from 'svelte/transition';
	import { LineChart, Layers, CircleDollarSign, Tags, CalendarRange } from '@lucide/svelte';
	import { getSubscriptions, getCategories, type Sub, type Category } from '$lib/api';
	import { lifecycle, money, advanceCycle, monthlyCents } from '$lib/format';
	import { i18n, locale } from '$lib/i18n.svelte';
	import { DUEO_COLORS } from '$lib/icons';
	import DonutChart from '$lib/DonutChart.svelte';

	let subs = $state<Sub[]>([]);
	let categories = $state<Category[]>([]);
	let loading = $state(true);

	onMount(async () => {
		const res = await getSubscriptions();
		if (res.status === 401) return goto('/login');
		if (res.ok) subs = await res.json();
		const cats = await getCategories();
		if (cats.ok) categories = await cats.json();
		loading = false;
	});

	const catById = $derived(new Map(categories.map((c) => [c.id, c])));
	const monthlyOf = monthlyCents;

	// Coste mensual agrupado por moneda (R2: sin conversión).
	const byCurrency = $derived.by(() => {
		const m = new Map<string, number>();
		for (const s of subs) m.set(s.currency, (m.get(s.currency) ?? 0) + monthlyOf(s));
		return [...m.entries()]
			.map(([currency, monthly]) => ({ currency, monthly }))
			.sort((a, b) => b.monthly - a.monthly);
	});

	// Moneda dominante (más suscripciones): base para los desgloses por categoría/top.
	const dominantCur = $derived.by(() => {
		const c = new Map<string, number>();
		for (const s of subs) c.set(s.currency, (c.get(s.currency) ?? 0) + 1);
		return [...c.entries()].sort((a, b) => b[1] - a[1])[0]?.[0] ?? 'USD';
	});
	const otherCurrencies = $derived(byCurrency.length > 1);

	const inDominant = $derived(subs.filter((s) => s.currency === dominantCur));

	// Gasto mensual por categoría (en la moneda dominante).
	const byCategory = $derived.by(() => {
		const m = new Map<number | 'none', number>();
		for (const s of inDominant) {
			const k = s.category_id ?? 'none';
			m.set(k, (m.get(k) ?? 0) + monthlyOf(s));
		}
		const rows = [...m.entries()].map(([key, monthly]) => {
			const cat = key === 'none' ? null : catById.get(key as number);
			return {
				key,
				name: cat?.name ?? i18n.t('ins.noCategory'),
				color: cat?.color ?? null,
				monthly
			};
		});
		rows.sort((a, b) => b.monthly - a.monthly);
		// color: el de la categoría o uno de la paleta Dueo por posición.
		return rows.map((r, i) => ({ ...r, color: r.color ?? DUEO_COLORS[i % DUEO_COLORS.length] }));
	});

	// Gasto PROYECTADO de los próximos 6 meses (cashflow): simula cada recurrencia
	// y suma por mes. En la moneda dominante.
	const months = $derived.by(() => {
		const now = new Date();
		return Array.from({ length: 6 }, (_, i) => {
			const d = new Date(now.getFullYear(), now.getMonth() + i, 1);
			return {
				y: d.getFullYear(),
				m: d.getMonth(),
				label: d.toLocaleDateString(locale(), { month: 'short' })
			};
		});
	});
	const MAX_RECURRENCES = 400; // guard against a runaway custom cycle
	const projected = $derived.by(() => {
		const out = months.map(() => 0);
		const last = months[months.length - 1];
		const end = new Date(last.y, last.m + 1, 0); // last day of the final month
		const idxOf = (d: Date) =>
			months.findIndex((mm) => mm.y === d.getFullYear() && mm.m === d.getMonth());
		for (const s of inDominant) {
			let cur: string | null = s.due_date;
			let guard = 0;
			while (cur && guard < MAX_RECURRENCES) {
				const d = new Date(cur + 'T00:00:00');
				if (d > end) break;
				const idx = idxOf(d);
				if (idx >= 0) out[idx] += s.amount_cents;
				cur = advanceCycle(cur, s.cycle, s.cycle_days);
				guard++;
			}
		}
		return out;
	});
	const projMax = $derived(Math.max(1, ...projected));

	// Top suscripciones por gasto mensual (moneda dominante).
	const topSubs = $derived([...inDominant].sort((a, b) => monthlyOf(b) - monthlyOf(a)).slice(0, 6));
	const topMax = $derived(Math.max(1, ...topSubs.map(monthlyOf)));

	// Conteo por estado.
	const view = $derived(subs.map((s) => ({ ...s, ...lifecycle(s.start_date, s.due_date) })));
	const counts = $derived({
		total: subs.length,
		active: subs.filter((s) => s.status === 'active').length,
		expired: subs.filter((s) => s.status === 'expired').length,
		paused: subs.filter((s) => s.status === 'paused').length,
		soon: view.filter((s) => s.days >= 0 && s.days <= 7).length
	});
</script>

<div class="page">
	<header class="bar">
		<div class="title">
			<LineChart size={22} />
			<h1>{i18n.t('ins.title')}</h1>
		</div>
	</header>

	{#if loading}
		<div class="grid">
			{#each Array(4) as _, i (i)}<div
					class="skeleton"
					style="height:84px;border-radius:16px"
				></div>{/each}
		</div>
		<div class="skeleton" style="height:220px;border-radius:16px;margin-top:1rem"></div>
	{:else if subs.length === 0}
		<p class="muted">{i18n.t('ins.empty')}</p>
	{:else}
		<!-- KPIs -->
		<section class="grid" in:fly={{ y: 12, duration: 280 }}>
			<div class="kpi acrylic">
				<span class="klabel"><Layers size={14} /> {i18n.t('ins.subscriptions')}</span>
				<span class="kval tnum">{counts.total}</span>
				<span class="ksub"
					>{i18n.t('ins.activeExpired', { a: counts.active, e: counts.expired })}</span
				>
			</div>
			<div class="kpi acrylic">
				<span class="klabel"><CircleDollarSign size={14} /> {i18n.t('ins.monthly')}</span>
				{#each byCurrency as t (t.currency)}
					<span class="kval-sm tnum">{money(t.monthly, t.currency)}</span>
				{/each}
			</div>
			<div class="kpi acrylic">
				<span class="klabel"><CircleDollarSign size={14} /> {i18n.t('ins.annual')}</span>
				{#each byCurrency as t (t.currency)}
					<span class="kval-sm tnum">{money(t.monthly * 12, t.currency)}</span>
				{/each}
			</div>
			<div class="kpi acrylic">
				<span class="klabel">{i18n.t('ins.dueSoon')}</span>
				<span class="kval tnum">{counts.soon}</span>
				<span class="ksub">{i18n.t('ins.next7')}</span>
			</div>
		</section>

		<section class="cols">
			<!-- Donut por categoría -->
			<div class="card" in:fly={{ y: 12, duration: 280, delay: 60 }}>
				<div class="chead">
					<Tags size={16} />
					<h2>{i18n.t('ins.byCategory')}</h2>
					{#if otherCurrencies}<span class="note">{i18n.t('ins.only', { cur: dominantCur })}</span
						>{/if}
				</div>
				<div class="donutwrap">
					<DonutChart
						items={byCategory.map((r) => ({ label: r.name, value: r.monthly, color: r.color }))}
						centerTop={money(
							byCategory.reduce((a, r) => a + r.monthly, 0),
							dominantCur
						)}
						centerSub={i18n.t('ins.perMonth')}
					/>
					<ul class="legend">
						{#each byCategory as r (r.key)}
							<li>
								<span class="sw" style="background:{r.color}"></span>
								<span class="lname" title={r.name}>{r.name}</span>
								<span class="lval tnum">{money(r.monthly, dominantCur)}</span>
							</li>
						{/each}
					</ul>
				</div>
			</div>

			<!-- Top por gasto -->
			<div class="card" in:fly={{ y: 12, duration: 280, delay: 120 }}>
				<div class="chead">
					<CircleDollarSign size={16} />
					<h2>{i18n.t('ins.top')}</h2>
					{#if otherCurrencies}<span class="note">{i18n.t('ins.only', { cur: dominantCur })}</span
						>{/if}
				</div>
				<div class="bars">
					{#each topSubs as s (s.id)}
						<div class="barrow">
							<span class="blabel" title={s.name}>{s.name}</span>
							<div class="btrack">
								<div
									class="bfill"
									style="width:{(monthlyOf(s) / topMax) * 100}%; --bc:var(--brand)"
								></div>
							</div>
							<span class="bval tnum">{money(monthlyOf(s), dominantCur)}</span>
						</div>
					{/each}
				</div>
			</div>
		</section>

		<!-- Gasto proyectado 6 meses (cashflow) -->
		<div class="card full" in:fly={{ y: 12, duration: 280, delay: 180 }}>
			<div class="chead">
				<CalendarRange size={16} />
				<h2>{i18n.t('ins.projected')}</h2>
				{#if otherCurrencies}<span class="note">{i18n.t('ins.only', { cur: dominantCur })}</span
					>{/if}
			</div>
			<div class="vbars">
				{#each months as mm, i (mm.label + i)}
					<div class="vcol">
						<span class="vval tnum">{money(projected[i], dominantCur)}</span>
						<div class="vtrack">
							<div class="vfill" style="height:{(projected[i] / projMax) * 100}%"></div>
						</div>
						<span class="vlabel">{mm.label}</span>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>

<style>
	.bar {
		margin-bottom: var(--gap-section);
	}
	.title {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		color: var(--text);
	}
	.title h1 {
		margin: 0;
		font-size: 1.3rem;
	}
	.muted {
		color: var(--text-muted);
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: var(--gap-grid);
	}
	.kpi {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 1rem 1.2rem;
		border-radius: var(--radius-lg, 16px);
	}
	.klabel {
		display: flex;
		align-items: center;
		gap: 0.35rem;
		font-size: 0.78rem;
		color: var(--text-muted);
	}
	.kval {
		font-size: 1.6rem;
		font-weight: 750;
	}
	.kval-sm {
		font-size: 1.05rem;
		font-weight: 700;
		line-height: 1.25;
	}
	.ksub {
		font-size: 0.74rem;
		color: var(--text-muted);
	}
	.cols {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--gap-grid);
		margin-top: var(--gap-section);
	}
	.card {
		padding: 1.2rem 1.3rem;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg, 16px);
	}
	.chead {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 1rem;
		color: var(--text);
	}
	.chead h2 {
		margin: 0;
		font-size: 1rem;
		flex: 1;
	}
	.note {
		font-size: 0.66rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--text-2);
		background: color-mix(in srgb, var(--text-muted) 16%, transparent);
		padding: 2px 7px;
		border-radius: 999px;
	}
	.bars {
		display: flex;
		flex-direction: column;
		gap: 0.7rem;
	}
	.barrow {
		display: grid;
		grid-template-columns: minmax(80px, 1fr) 2fr auto;
		align-items: center;
		gap: 0.6rem;
	}
	.blabel {
		font-size: 0.82rem;
		color: var(--text-2);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.btrack {
		height: 10px;
		border-radius: 999px;
		background: color-mix(in srgb, var(--text-muted) 16%, transparent);
		overflow: hidden;
	}
	.bfill {
		height: 100%;
		border-radius: 999px;
		background: var(--bc);
		min-width: 4px;
		transition: width 0.5s cubic-bezier(0.2, 0.8, 0.2, 1);
	}
	.bval {
		font-size: 0.82rem;
		font-weight: 650;
		color: var(--text);
	}

	/* donut + leyenda */
	.donutwrap {
		display: flex;
		align-items: center;
		gap: 1.2rem;
	}
	.legend {
		list-style: none;
		margin: 0;
		padding: 0;
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		min-width: 0;
	}
	.legend li {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.84rem;
	}
	.sw {
		width: 11px;
		height: 11px;
		border-radius: 3px;
		flex: none;
	}
	.lname {
		flex: 1;
		color: var(--text-2);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.lval {
		font-weight: 650;
		color: var(--text);
	}

	/* proyección (barras verticales) */
	.full {
		margin-top: var(--gap-grid);
	}
	.vbars {
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: 0.6rem;
		align-items: end;
		height: 180px;
	}
	.vcol {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: flex-end;
		gap: 0.4rem;
		height: 100%;
	}
	.vval {
		font-size: 0.7rem;
		font-weight: 650;
		color: var(--text-2);
		white-space: nowrap;
	}
	.vtrack {
		width: 100%;
		max-width: 54px;
		flex: 1;
		display: flex;
		align-items: flex-end;
	}
	.vfill {
		width: 100%;
		min-height: 4px;
		border-radius: 8px 8px 0 0;
		background: linear-gradient(180deg, var(--brand), var(--brand-2));
		transition: height 0.6s cubic-bezier(0.2, 0.8, 0.2, 1);
	}
	.vlabel {
		font-size: 0.74rem;
		color: var(--text-muted);
		text-transform: capitalize;
	}
	@media (max-width: 760px) {
		.grid {
			grid-template-columns: 1fr 1fr;
		}
		.cols {
			grid-template-columns: 1fr;
		}
	}
	/* Móvil: los importes (nowrap) ensanchaban las columnas y el grid de la
	   proyección se desbordaba → minmax(0,1fr) lo contiene y el valor parte en
	   dos líneas (código / cantidad) con fuente menor. */
	@media (max-width: 640px) {
		.vbars {
			grid-template-columns: repeat(6, minmax(0, 1fr));
			gap: 0.3rem;
		}
		.vval {
			white-space: normal;
			text-align: center;
			font-size: 0.58rem;
			line-height: 1.1;
		}
		.vlabel {
			font-size: 0.66rem;
		}
	}

	/* Móvil estrecho: el donut (168px) + leyenda no caben en fila → apilados. */
	@media (max-width: 480px) {
		.donutwrap {
			flex-direction: column;
			align-items: center;
			gap: 1rem;
		}
		.legend {
			width: 100%;
		}
	}
</style>
