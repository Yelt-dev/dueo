<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { fly, fade } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import { Plus, Receipt, Inbox, X, Search } from '@lucide/svelte';
	import HorizonTimeline from '$lib/HorizonTimeline.svelte';
	import SubscriptionRow from '$lib/SubscriptionRow.svelte';
	import RowSkeleton from '$lib/RowSkeleton.svelte';
	import AddSubscriptionModal from '$lib/AddSubscriptionModal.svelte';
	import {
		getSubscriptions,
		updateSubscription,
		deleteSubscription,
		getCategories,
		me,
		type Sub,
		type Category
	} from '$lib/api';
	import { lifecycle, money, advanceCycle, monthlyCents } from '$lib/format';
	import { i18n, cycleLabel } from '$lib/i18n.svelte';
	import { resolveSubVisual } from '$lib/icons';
	import { ensureBrands } from '$lib/brandcat.svelte';

	let subs = $state<Sub[]>([]);
	let categories = $state<Category[]>([]);
	let userCurrency = $state('USD');
	let loading = $state(true);
	let showAdd = $state(false);
	let editing = $state<Sub | null>(null);
	let highlightedId = $state<number | null>(null);
	let filterId = $state<number | null>(null); // filtro rápido por un item (clic Horizonte)
	let hlTimer: ReturnType<typeof setTimeout>;

	// Controles de la lista.
	let search = $state('');
	let statusFilter = $state<'all' | 'active' | 'expired' | 'paused'>('all');
	let catFilter = $state<'all' | 'none' | number>('all');
	let sortBy = $state<'due' | 'amount' | 'name'>('due');
	const STATUSES = $derived([
		{ k: 'all', label: i18n.t('dash.statusAll') },
		{ k: 'active', label: i18n.t('dash.statusActive') },
		{ k: 'expired', label: i18n.t('dash.statusExpired') },
		{ k: 'paused', label: i18n.t('dash.statusPaused') }
	] as const);
	const anyFilter = $derived(
		search.trim() !== '' || statusFilter !== 'all' || catFilter !== 'all' || sortBy !== 'due'
	);
	function clearControls() {
		search = '';
		statusFilter = 'all';
		catFilter = 'all';
		sortBy = 'due';
	}

	// Clic en un marcador del Horizonte → FILTRA la lista a ese item y lo resalta.
	function focusSub(id: number) {
		filterId = id;
		highlightedId = id;
		clearTimeout(hlTimer);
		hlTimer = setTimeout(() => (highlightedId = null), 1800);
		requestAnimationFrame(() =>
			document.getElementById(`sub-${id}`)?.scrollIntoView({ behavior: 'smooth', block: 'center' })
		);
	}

	function clearFilter() {
		filterId = null;
		highlightedId = null;
	}

	// Mapa id→categoría para resolver nombre/color por fila en O(1).
	const catById = $derived(new Map(categories.map((c) => [c.id, c])));

	// Insertar o reemplazar una sub guardada (alta o edición).
	function onSaved(saved: Sub) {
		subs = subs.some((x) => x.id === saved.id)
			? subs.map((x) => (x.id === saved.id ? saved : x))
			: [...subs, saved];
	}

	function editSub(s: Sub) {
		editing = s;
		showAdd = true;
	}

	// Transient error toast for mutations that fail unexpectedly (not a 404).
	let flash = $state('');
	let flashTimer: ReturnType<typeof setTimeout>;
	function flashError() {
		flash = i18n.t('common.actionError');
		clearTimeout(flashTimer);
		flashTimer = setTimeout(() => (flash = ''), 4000);
	}

	async function del(s: Sub) {
		const res = await deleteSubscription(s.id);
		if (res.ok || res.status === 404) subs = subs.filter((x) => x.id !== s.id);
		else flashError();
	}

	async function renew(s: Sub) {
		const due = advanceCycle(s.due_date, s.cycle, s.cycle_days);
		if (!due) return;
		const res = await updateSubscription(s.id, {
			start_date: s.due_date,
			due_date: due,
			status: 'active'
		});
		if (res.ok) onSaved(await res.json());
		else flashError();
	}

	// Derivados: enriquecemos cada sub con progreso/días, y calculamos KPIs.
	const view = $derived(
		subs.map((s) => ({
			...s,
			...lifecycle(s.start_date, s.due_date),
			cyc: cycleLabel(s.cycle, s.cycle_days)
		}))
	);
	const horizon = $derived(
		view.map((s) => ({
			id: s.id,
			name: s.name,
			days: s.days,
			progress: s.progress,
			icon: s.icon,
			color: s.color
		}))
	);
	// Lista resultante. El filtro por clic (filterId) MANDA; si no, aplica controles.
	// KPIs y Horizonte siempre con todo (vista general).
	const shownView = $derived.by(() => {
		if (filterId) return view.filter((s) => s.id === filterId);
		let r = view;
		const q = search.trim().toLowerCase();
		if (q) r = r.filter((s) => s.name.toLowerCase().includes(q));
		if (statusFilter !== 'all') r = r.filter((s) => s.status === statusFilter);
		if (catFilter !== 'all')
			r = r.filter((s) => (catFilter === 'none' ? !s.category_id : s.category_id === catFilter));
		const by = sortBy;
		return [...r].sort((a, b) =>
			by === 'amount'
				? b.amount_cents - a.amount_cents
				: by === 'name'
					? a.name.localeCompare(b.name)
					: a.days - b.days
		);
	});
	const filterName = $derived(
		filterId ? (subs.find((s) => s.id === filterId)?.name ?? null) : null
	);
	// R2: en el MVP NO se convierte entre monedas. Agrupamos el coste mensual
	// POR moneda (normalización por ciclo: anual → /12; resto tal cual).
	const monthlyByCurrency = $derived.by(() => {
		const m = new Map<string, number>();
		for (const s of view) {
			m.set(s.currency, (m.get(s.currency) ?? 0) + monthlyCents(s));
		}
		return [...m.entries()]
			.map(([currency, monthly]) => ({ currency, monthly }))
			.sort((a, b) => b.monthly - a.monthly); // moneda dominante primero
	});
	const multiCurrency = $derived(monthlyByCurrency.length > 1);
	const nextDays = $derived(view.length ? Math.min(...view.map((s) => s.days)) : 0);

	onMount(async () => {
		try {
			const res = await getSubscriptions();
			if (res.status === 401) return goto('/login');
			subs = await res.json();
			// Solo descargamos el catálogo de marcas si alguna sub usa una marca explícita.
			if (subs.some((s) => s.icon?.startsWith('si:'))) ensureBrands();
			const cats = await getCategories();
			if (cats.ok) categories = await cats.json();
			const m = await me();
			if (m.ok) userCurrency = (await m.json()).default_currency ?? 'USD';
		} catch {
			// backend caído: dejamos lista vacía
		} finally {
			loading = false;
		}
	});
</script>

<div class="page">
	{#if flash}
		<div class="flash" role="alert" transition:fly={{ y: 12, duration: 200 }}>{flash}</div>
	{/if}
	<header class="bar">
		<div class="heading">
			<h1>{i18n.t('dash.title')}</h1>
			{#if view.length}
				<span class="tag"
					>{view.length}
					{view.length === 1 ? i18n.t('dash.serviceOne') : i18n.t('dash.serviceMany')}</span
				>
			{/if}
		</div>
		<button
			class="add"
			onclick={() => {
				editing = null;
				showAdd = true;
			}}><Plus size={16} /> {i18n.t('dash.add')}</button
		>
	</header>

	{#if loading}
		<section class="kpis">
			{#each Array(3) as _, i (i)}
				<div class="kpi acrylic">
					<span class="skeleton" style="width:90px;height:10px"></span>
					<span class="skeleton" style="width:120px;height:22px;margin-top:8px"></span>
				</div>
			{/each}
		</section>
		<div class="skeleton hzskel"></div>
		<section class="list">
			{#each Array(4) as _, i (i)}<RowSkeleton />{/each}
		</section>
	{:else if view.length === 0}
		<div class="empty">
			<Inbox size={40} />
			<h2>{i18n.t('dash.emptyTitle')}</h2>
			<p>{i18n.t('dash.emptyText')}</p>
			<button
				class="add"
				onclick={() => {
					editing = null;
					showAdd = true;
				}}><Plus size={16} /> {i18n.t('dash.addFirst')}</button
			>
		</div>
	{:else}
		<section class="kpis">
			<div class="kpi acrylic">
				<span class="klabel">
					{i18n.t('dash.monthly')}
					{#if multiCurrency}<span class="note">{i18n.t('dash.noConversion')}</span>{/if}
				</span>
				{#if multiCurrency}
					<div class="kmulti">
						{#each monthlyByCurrency as t (t.currency)}
							<span class="kval-sm tnum">{money(t.monthly, t.currency)}</span>
						{/each}
					</div>
				{:else}
					<span class="kval tnum"
						>{money(monthlyByCurrency[0]?.monthly ?? 0, monthlyByCurrency[0]?.currency)}</span
					>
				{/if}
			</div>
			<div class="kpi acrylic">
				<span class="klabel">
					{i18n.t('dash.annual')}
					{#if multiCurrency}<span class="note">{i18n.t('dash.noConversion')}</span>{/if}
				</span>
				{#if multiCurrency}
					<div class="kmulti">
						{#each monthlyByCurrency as t (t.currency)}
							<span class="kval-sm tnum">{money(t.monthly * 12, t.currency)}</span>
						{/each}
					</div>
				{:else}
					<span class="kval tnum"
						>{money(
							(monthlyByCurrency[0]?.monthly ?? 0) * 12,
							monthlyByCurrency[0]?.currency
						)}</span
					>
				{/if}
			</div>
			<div class="kpi acrylic">
				<span class="klabel">{i18n.t('dash.nextDue')}</span>
				<span class="kval tnum">{i18n.t('dash.inDays', { n: nextDays })}</span>
			</div>
		</section>

		<HorizonTimeline items={horizon} onselect={focusSub} />

		{#if filterName}
			<div class="filterbar">
				<span>{i18n.t('dash.showing')} <strong>{filterName}</strong></span>
				<button onclick={clearFilter} aria-label={i18n.t('dash.removeFilter')}
					>{i18n.t('dash.showAll')} <X size={14} /></button
				>
			</div>
		{:else}
			<div class="controls">
				<div class="searchbox">
					<Search size={15} />
					<input bind:value={search} placeholder={i18n.t('dash.searchPlaceholder')} />
				</div>
				<div class="chips">
					{#each STATUSES as st (st.k)}
						<button class:on={statusFilter === st.k} onclick={() => (statusFilter = st.k)}
							>{st.label}</button
						>
					{/each}
				</div>
				<select bind:value={catFilter} aria-label={i18n.t('dash.categoryAria')}>
					<option value="all">{i18n.t('dash.allCategories')}</option>
					{#each categories as c (c.id)}
						<option value={c.id}>{c.name}</option>
					{/each}
					<option value="none">{i18n.t('dash.noCategory')}</option>
				</select>
				<select bind:value={sortBy} aria-label={i18n.t('dash.sortAria')}>
					<option value="due">{i18n.t('dash.sortDue')}</option>
					<option value="amount">{i18n.t('dash.sortAmount')}</option>
					<option value="name">{i18n.t('dash.sortName')}</option>
				</select>
				{#if anyFilter}
					<button
						class="clear"
						onclick={clearControls}
						in:fly={{ x: 8, duration: 180 }}
						out:fade={{ duration: 120 }}
					>
						{i18n.t('common.clear')}
						<X size={13} />
					</button>
				{/if}
			</div>
		{/if}

		<section class="list">
			{#if shownView.length === 0}
				<p class="noresults" transition:fade={{ duration: 160 }}>
					{i18n.t('dash.noResults')}
				</p>
			{/if}
			{#each shownView as s, i (s.id)}
				{@const cat = s.category_id ? catById.get(s.category_id) : null}
				{@const vis = resolveSubVisual(s, cat?.color)}
				<div
					animate:flip={{ duration: 280 }}
					in:fly={{ y: 12, duration: 320, delay: Math.min(i, 8) * 40 }}
					out:fade={{ duration: 140 }}
				>
					<SubscriptionRow
						domId={`sub-${s.id}`}
						highlighted={highlightedId === s.id}
						name={s.name}
						icon={Receipt}
						iconDef={vis.def}
						brand={vis.brand}
						catLabel={cat ? cat.name : vis.brand ? vis.brand.label : i18n.t('dash.noCategory')}
						chipColor={vis.color}
						amountLabel={money(s.amount_cents, s.currency)}
						cycle={s.cyc}
						days={s.days}
						progress={s.progress}
						status={s.status}
						paymentMode={s.payment_mode}
						canRenew={s.cycle !== 'once'}
						onrenew={() => renew(s)}
						onedit={() => editSub(s)}
						ondelete={() => del(s)}
					/>
				</div>
			{/each}
		</section>
	{/if}

	<AddSubscriptionModal
		open={showAdd}
		{editing}
		{categories}
		defaultCurrency={userCurrency}
		onclose={() => {
			showAdd = false;
			editing = null;
		}}
		onsaved={onSaved}
	/>
</div>

<style>
	.bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		margin-bottom: var(--gap-section);
	}
	.heading {
		display: flex;
		flex-direction: column;
		gap: 0.15rem;
	}
	.heading h1 {
		margin: 0;
		font-size: 1.3rem;
	}
	.tag {
		font-size: 0.8rem;
		color: var(--text-muted);
	}
	.add {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 40px;
		padding: 0 1rem;
		border: none;
		border-radius: 12px;
		font-weight: 650;
		font-size: 0.9rem;
		color: white;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
		cursor: pointer;
		transition: transform 0.15s;
	}
	.add:hover {
		transform: translateY(-1px);
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		text-align: center;
		padding: 4rem 1rem;
		color: var(--text-muted);
	}
	.empty h2 {
		margin: 0.5rem 0 0;
		color: var(--text);
		font-size: 1.15rem;
	}
	.empty p {
		margin: 0 0 1rem;
	}
	.kpis {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--gap-grid);
		margin-bottom: var(--gap-section);
	}
	.hzskel {
		height: 200px;
		border-radius: var(--radius-xl, 20px);
		margin-bottom: var(--gap-section);
	}
	.kpi {
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
		padding: 1rem 1.2rem;
		border-radius: var(--radius-lg, 16px);
	}
	.klabel {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.78rem;
		color: var(--text-muted);
	}
	.note {
		font-size: 0.62rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		color: var(--text-2);
		background: color-mix(in srgb, var(--text-muted) 16%, transparent);
		padding: 1px 6px;
		border-radius: 999px;
	}
	.kval {
		font-size: 1.35rem;
		font-weight: 750;
	}
	/* multi-moneda: cada moneda en su línea (sin convertir, R2) */
	.kmulti {
		display: flex;
		flex-direction: column;
		gap: 0.1rem;
	}
	.kval-sm {
		font-size: 1.05rem;
		font-weight: 700;
		line-height: 1.25;
	}
	.filterbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.75rem;
		margin-bottom: var(--gap-list);
		padding: 0.6rem 1rem;
		border: 1px solid color-mix(in srgb, var(--brand) 35%, var(--border));
		border-radius: var(--radius-md, 12px);
		background: color-mix(in srgb, var(--brand) 8%, transparent);
		font-size: 0.88rem;
		color: var(--text-2);
	}
	.filterbar strong {
		color: var(--text);
	}
	.filterbar button {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 4px 10px;
		border-radius: 999px;
		border: 1px solid var(--border);
		background: var(--surface);
		color: var(--text-2);
		font-size: 0.82rem;
		font-weight: 600;
		cursor: pointer;
	}
	.filterbar button:hover {
		color: var(--text);
		border-color: var(--border-strong);
	}
	/* controles de la lista */
	.controls {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: var(--gap-list);
	}
	.searchbox {
		display: flex;
		align-items: center;
		gap: 0.45rem;
		padding: 0 0.7rem;
		border: 1px solid var(--border);
		border-radius: 10px;
		background: var(--surface);
		color: var(--text-muted);
		flex: 1 1 200px;
	}
	.searchbox input {
		border: none;
		background: transparent;
		padding: 0.5rem 0;
		flex: 1;
		min-width: 0;
		color: var(--text);
		font-size: 0.88rem;
	}
	.searchbox input:focus-visible {
		outline: none;
	}
	.chips {
		display: flex;
		gap: 3px;
		padding: 3px;
		border-radius: 10px;
		background: var(--surface-2);
		border: 1px solid var(--border);
	}
	.chips button {
		border: none;
		background: transparent;
		color: var(--text-2);
		font-size: 0.8rem;
		font-weight: 600;
		padding: 5px 10px;
		border-radius: 7px;
		cursor: pointer;
		transition:
			color 0.12s,
			background 0.12s;
	}
	.chips button:hover {
		color: var(--text);
	}
	.chips button.on {
		color: white;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
	}
	.controls select {
		padding: 0.5rem 0.6rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--surface);
		color: var(--text);
		font-size: 0.85rem;
		cursor: pointer;
	}
	.controls .clear {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 0.5rem 0.7rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: transparent;
		color: var(--text-2);
		font-size: 0.82rem;
		font-weight: 600;
		cursor: pointer;
	}
	.controls .clear:hover {
		color: var(--danger);
		border-color: color-mix(in srgb, var(--danger) 40%, transparent);
	}
	.noresults {
		margin: 0;
		padding: 1.5rem;
		text-align: center;
		color: var(--text-muted);
		font-size: 0.88rem;
	}
	.list {
		display: flex;
		flex-direction: column;
		gap: var(--gap-list);
	}
	.flash {
		position: fixed;
		left: 50%;
		bottom: 1.2rem;
		transform: translateX(-50%);
		z-index: 50;
		padding: 0.7rem 1rem;
		border-radius: 12px;
		background: var(--surface);
		border: 1px solid color-mix(in srgb, var(--danger) 45%, var(--border));
		color: var(--text);
		font-size: 0.85rem;
		box-shadow: 0 14px 36px -14px rgba(0, 0, 0, 0.6);
	}
	@media (max-width: 640px) {
		.kpis {
			grid-template-columns: 1fr;
		}
		/* Los chips de estado se salían en una sola fila → 2×2 a todo el ancho. */
		.chips {
			display: grid;
			grid-template-columns: 1fr 1fr;
			flex-basis: 100%;
			gap: 3px;
		}
		.chips button {
			text-align: center;
		}
		/* Categorías y "ordenar por": cada uno a todo el ancho. */
		.controls select {
			flex: 1 1 100%;
			width: 100%;
		}
	}
</style>
