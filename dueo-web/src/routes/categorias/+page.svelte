<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Plus, Tags, Pencil, Trash2, Check, X } from '@lucide/svelte';
	import {
		getCategories,
		createCategory,
		updateCategory,
		deleteCategory,
		type Category
	} from '$lib/api';
	import { i18n } from '$lib/i18n.svelte';

	// Suggested palette (live Design System tokens / common brands).
	const SWATCHES = [
		'#E50914',
		'#1DB954',
		'#3b82f6',
		'#22c55e',
		'#f59e0b',
		'#a855f7',
		'#ec4899',
		'#14b8a6',
		'#ef4444',
		'#64748b'
	];

	let categories = $state<Category[]>([]);
	let loading = $state(true);

	// Inline edit/create: id being edited (null = none, 0 = new), with a draft.
	let editingId = $state<number | null>(null);
	let draftName = $state('');
	let draftColor = $state(SWATCHES[2]);
	let confirmId = $state<number | null>(null);
	let busy = $state(false);

	onMount(async () => {
		const res = await getCategories();
		if (res.status === 401) return goto('/login');
		if (res.ok && res.data) categories = res.data;
		loading = false;
	});

	function startNew() {
		editingId = 0;
		draftName = '';
		draftColor = SWATCHES[2];
	}

	function startEdit(c: Category) {
		editingId = c.id;
		draftName = c.name;
		draftColor = c.color || SWATCHES[2];
	}

	function cancel() {
		editingId = null;
		draftName = '';
	}

	async function save() {
		const name = draftName.trim();
		if (!name || busy) return;
		busy = true;
		try {
			if (editingId === 0) {
				const res = await createCategory({
					name,
					color: draftColor,
					sort_order: categories.length
				});
				if (!res.ok || !res.data) return; // keep the draft so the user can retry
				categories = [...categories, res.data];
			} else if (editingId != null) {
				const res = await updateCategory(editingId, { name, color: draftColor });
				if (!res.ok || !res.data) return;
				const updated = res.data;
				categories = categories.map((c) => (c.id === updated.id ? updated : c));
			}
			cancel(); // only on success
		} finally {
			busy = false;
		}
	}

	async function del(c: Category) {
		const res = await deleteCategory(c.id);
		if (res.ok || res.status === 404) categories = categories.filter((x) => x.id !== c.id);
		confirmId = null;
	}
</script>

<div class="page">
	<header class="bar">
		<div class="title">
			<Tags size={22} />
			<h1>{i18n.t('cat.title')}</h1>
		</div>
		{#if editingId === null}
			<button class="add" onclick={startNew}><Plus size={16} /> {i18n.t('cat.new')}</button>
		{/if}
	</header>

	<p class="hint">{i18n.t('cat.hint')}</p>

	<!-- Same form for create (editingId===0) and edit; only the placeholder differs. -->
	{#snippet categoryForm(placeholder: string)}
		<form class="card acrylic editing" onsubmit={(e) => (e.preventDefault(), save())}>
			<span class="swatch" style="--cc:{draftColor}"></span>
			<!-- svelte-ignore a11y_autofocus -->
			<input class="namein" bind:value={draftName} {placeholder} autofocus />
			<div class="palette">
				{#each SWATCHES as s (s)}
					<button
						type="button"
						class="dot"
						class:on={draftColor === s}
						style="--cc:{s}"
						aria-label={s}
						onclick={() => (draftColor = s)}
					></button>
				{/each}
			</div>
			<div class="rowacts">
				<button type="button" class="ic" onclick={cancel} aria-label={i18n.t('common.cancel')}
					><X size={16} /></button
				>
				<button type="submit" class="ic ok" disabled={busy} aria-label={i18n.t('common.save')}
					><Check size={16} /></button
				>
			</div>
		</form>
	{/snippet}

	{#if loading}
		<section class="list">
			{#each Array(3) as _, i (i)}
				<article class="card row">
					<span class="skeleton" style="width:22px;height:22px;border-radius:7px;flex:none"></span>
					<span class="skeleton" style="width:130px;height:13px"></span>
				</article>
			{/each}
		</section>
	{:else}
		<section class="list">
			{#if editingId === 0}
				{@render categoryForm(i18n.t('cat.nameFull'))}
			{/if}

			{#each categories as c (c.id)}
				{#if editingId === c.id}
					{@render categoryForm(i18n.t('cat.name'))}
				{:else}
					<article class="card row">
						<span class="swatch" style="--cc:{c.color || 'var(--brand)'}"></span>
						<span class="cname">{c.name}</span>
						<div class="rowacts">
							{#if confirmId === c.id}
								<button
									class="ic danger"
									onclick={() => del(c)}
									aria-label={i18n.t('cat.confirmDeleteAria')}
								>
									<Trash2 size={15} />
									{i18n.t('common.confirmDelete')}
								</button>
								<button
									class="ic"
									onclick={() => (confirmId = null)}
									aria-label={i18n.t('common.cancel')}><X size={15} /></button
								>
							{:else}
								<button class="ic" onclick={() => startEdit(c)} aria-label={i18n.t('common.edit')}
									><Pencil size={15} /></button
								>
								<button
									class="ic danger"
									onclick={() => (confirmId = c.id)}
									aria-label={i18n.t('common.delete')}><Trash2 size={15} /></button
								>
							{/if}
						</div>
					</article>
				{/if}
			{/each}

			{#if categories.length === 0 && editingId === null}
				<div class="empty">
					<Tags size={36} />
					<p>{i18n.t('cat.empty')}</p>
				</div>
			{/if}
		</section>
	{/if}
</div>

<style>
	.bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		margin-bottom: 0.4rem;
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
	.hint {
		margin: 0 0 1.5rem;
		color: var(--text-muted);
		font-size: 0.88rem;
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
	.list {
		display: flex;
		flex-direction: column;
		gap: var(--gap-list);
	}
	.card {
		display: flex;
		align-items: center;
		gap: 0.9rem;
		padding: 0.85rem 1.1rem;
		background: var(--surface);
		border: 1px solid var(--border);
		border-radius: var(--radius-lg, 16px);
	}
	.row {
		transition:
			border-color 0.15s,
			transform 0.15s;
	}
	.row:hover {
		transform: translateY(-1px);
		border-color: var(--border-strong);
	}
	.swatch {
		width: 22px;
		height: 22px;
		border-radius: 7px;
		flex: none;
		background: var(--cc);
		box-shadow:
			0 0 0 1px color-mix(in srgb, var(--cc) 40%, transparent),
			0 0 14px -3px var(--cc);
	}
	.cname {
		flex: 1;
		font-weight: 600;
		font-size: 0.95rem;
	}
	.namein {
		flex: 1;
		min-width: 0;
		padding: 0.5rem 0.7rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text);
		font-size: 0.92rem;
	}
	.namein:focus-visible {
		outline: 2px solid var(--brand);
		outline-offset: 1px;
		border-color: transparent;
	}
	.palette {
		display: flex;
		gap: 5px;
	}
	.dot {
		width: 18px;
		height: 18px;
		border-radius: 999px;
		border: 2px solid transparent;
		background: var(--cc);
		cursor: pointer;
		padding: 0;
		transition: transform 0.12s;
	}
	.dot:hover {
		transform: scale(1.15);
	}
	.dot.on {
		border-color: var(--text);
	}
	.rowacts {
		display: flex;
		align-items: center;
		gap: 0.35rem;
	}
	.ic {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		height: 32px;
		padding: 0 0.6rem;
		border-radius: 9px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text-2);
		font-size: 0.82rem;
		font-weight: 550;
		cursor: pointer;
		transition:
			color 0.15s,
			border-color 0.15s,
			background 0.15s;
	}
	.ic:hover {
		color: var(--text);
		border-color: var(--border-strong);
	}
	.ic.ok {
		color: white;
		border-color: transparent;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
	}
	.ic.danger:hover {
		color: var(--danger);
		border-color: color-mix(in srgb, var(--danger) 40%, transparent);
		background: color-mix(in srgb, var(--danger) 12%, transparent);
	}
	.ic:disabled {
		opacity: 0.6;
		cursor: default;
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.6rem;
		text-align: center;
		padding: 3rem 1rem;
		color: var(--text-muted);
	}

	/* Mobile: the edit form (input + 10-color palette + actions) doesn't fit on
	   one line → the name takes full width and the rest drops below and wraps. */
	@media (max-width: 560px) {
		.editing {
			flex-wrap: wrap;
		}
		.editing .namein {
			flex: 1 1 100%;
			order: -1;
		}
		.palette {
			flex-wrap: wrap;
		}
	}
</style>
