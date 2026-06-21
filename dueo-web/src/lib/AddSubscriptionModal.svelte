<script lang="ts">
	import { X } from '@lucide/svelte';
	import {
		createSubscription,
		updateSubscription,
		type Sub,
		type NewSub,
		type Category
	} from './api';
	import Icon from './Icon.svelte';
	import { resolveSubVisual } from './icons';
	import { i18n } from './i18n.svelte';
	import { fade, scale } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import IconColorPicker from './IconColorPicker.svelte';
	import ReminderEditor from './ReminderEditor.svelte';

	let {
		open = false,
		editing = null,
		categories = [],
		defaultCurrency = 'USD',
		onclose,
		onsaved
	}: {
		open?: boolean;
		editing?: Sub | null;
		categories?: Category[];
		defaultCurrency?: string;
		onclose?: () => void;
		onsaved?: (sub: Sub) => void;
	} = $props();

	const today = () => new Date().toISOString().slice(0, 10);

	// All form fields in one bag so reset/preload are a single assignment.
	function blankForm() {
		return {
			name: '',
			amount: '',
			currency: 'USD',
			cycle: 'monthly',
			cycleDays: '30',
			startDate: today(),
			dueDate: '',
			dueTouched: false,
			paymentMode: 'manual',
			categoryId: '',
			notes: '',
			icon: null as string | null,
			color: null as string | null
		};
	}
	function fromSub(s: Sub) {
		return {
			name: s.name,
			amount: (s.amount_cents / 100).toString(),
			currency: s.currency,
			cycle: s.cycle,
			cycleDays: String(s.cycle_days ?? 30),
			startDate: s.start_date,
			dueDate: s.due_date,
			dueTouched: true,
			paymentMode: s.payment_mode,
			categoryId: s.category_id ? String(s.category_id) : '',
			notes: s.notes ?? '',
			icon: s.icon ?? null,
			color: s.color ?? null
		};
	}

	let f = $state(blankForm());
	let error = $state('');
	let saving = $state(false);

	// What will render (explicit icon → brand-by-name → generic) + color.
	const preview = $derived(resolveSubVisual({ name: f.name, icon: f.icon, color: f.color }));

	// Offered currencies (ISO 4217): majors + LATAM. If the edited sub carries a
	// currency outside the list, prepend it so it isn't lost.
	const COMMON_CURRENCIES = [
		'USD',
		'EUR',
		'GBP',
		'MXN',
		'ARS',
		'COP',
		'CLP',
		'PEN',
		'BRL',
		'UYU',
		'BOB',
		'PYG',
		'VES',
		'CRC',
		'GTQ',
		'DOP',
		'CAD',
		'JPY',
		'CNY',
		'CHF',
		'AUD'
	];
	const currencyOptions = $derived(
		COMMON_CURRENCIES.includes(f.currency) || !f.currency
			? COMMON_CURRENCIES
			: [f.currency, ...COMMON_CURRENCIES]
	);

	// Autosuggest due date from start + cycle, unless the user edited it.
	$effect(() => {
		const s = f.startDate;
		const c = f.cycle;
		const n = parseInt(f.cycleDays) || 0;
		if (f.dueTouched || !s) return;
		const d = new Date(s + 'T00:00:00');
		if (c === 'monthly') d.setMonth(d.getMonth() + 1);
		else if (c === 'yearly') d.setFullYear(d.getFullYear() + 1);
		else if (c === 'custom') d.setDate(d.getDate() + n);
		else return; // once
		f.dueDate = d.toISOString().slice(0, 10);
	});

	// Preload ONCE on the open transition, not reactively: re-rendering the
	// `editing` object must not clobber edits already in progress.
	let lastOpened = $state(false);
	$effect(() => {
		if (open && !lastOpened) {
			lastOpened = true;
			// New sub: seed with the user's main currency
			f = editing ? fromSub(editing) : { ...blankForm(), currency: defaultCurrency };
		} else if (!open && lastOpened) {
			lastOpened = false;
		}
	});

	function close() {
		f = blankForm();
		error = '';
		onclose?.();
	}

	async function submit(e: Event) {
		e.preventDefault();
		error = '';
		const cents = Math.round(parseFloat(f.amount.replace(',', '.')) * 100);
		if (!f.name.trim()) return (error = i18n.t('modal.errName'));
		if (isNaN(cents) || cents < 0) return (error = i18n.t('modal.errAmount'));
		if (!f.startDate || !f.dueDate) return (error = i18n.t('modal.errDates'));

		saving = true;
		try {
			const body: NewSub = {
				name: f.name.trim(),
				amount_cents: cents,
				currency: f.currency,
				cycle: f.cycle,
				cycle_days: f.cycle === 'custom' ? parseInt(f.cycleDays) || null : null,
				start_date: f.startDate,
				due_date: f.dueDate,
				category_id: f.categoryId ? Number(f.categoryId) : null,
				payment_mode: f.paymentMode,
				notes: f.notes.trim() || null,
				icon: f.icon,
				color: f.color
			};
			const res = editing
				? await updateSubscription(editing.id, body)
				: await createSubscription(body);
			if (!res.ok) return (error = editing ? i18n.t('modal.errSave') : i18n.t('modal.errCreate'));
			if (res.data) onsaved?.(res.data);
			close();
		} catch {
			error = i18n.t('common.connError');
		} finally {
			saving = false;
		}
	}
</script>

{#snippet paymentSelect()}
	<label>
		{i18n.t('modal.payment')}
		<select bind:value={f.paymentMode}>
			<option value="manual">{i18n.t('modal.paymentManual')}</option>
			<option value="auto">{i18n.t('modal.paymentAuto')}</option>
		</select>
	</label>
{/snippet}

<svelte:window onkeydown={(e) => open && e.key === 'Escape' && close()} />

{#if open}
	<!-- Close only when the click lands on the backdrop, not inside the card (so the
	     form needs no onclick that would trigger a11y warnings). -->
	<div
		class="backdrop"
		onclick={(e) => e.target === e.currentTarget && close()}
		role="presentation"
		transition:fade={{ duration: 160 }}
	>
		<form
			class="card acrylic"
			onsubmit={submit}
			transition:scale={{ start: 0.96, opacity: 0, duration: 200, easing: cubicOut }}
		>
			<header>
				<span class="chip has" style="--cc:{preview.color}">
					{#if preview.def}
						<Icon def={preview.def} size={18} />
					{:else if preview.brand}
						<svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"
							><path d={preview.brand.path} /></svg
						>
					{/if}
				</span>
				<h2>{f.name.trim() || i18n.t('modal.newSub')}</h2>
				<button type="button" class="x" onclick={close} aria-label={i18n.t('common.close')}
					><X size={18} /></button
				>
			</header>

			<label>
				{i18n.t('modal.name')}
				<input bind:value={f.name} placeholder={i18n.t('modal.namePlaceholder')} required />
			</label>

			<IconColorPicker bind:icon={f.icon} bind:color={f.color} />

			<div class="grid2">
				<label>
					{i18n.t('modal.amount')}
					<input bind:value={f.amount} inputmode="decimal" placeholder="9.99" />
				</label>
				<label>
					{i18n.t('modal.currency')}
					<select bind:value={f.currency}>
						{#each currencyOptions as c (c)}
							<option value={c}>{c}</option>
						{/each}
					</select>
				</label>
			</div>

			<div class="grid2">
				<label>
					{i18n.t('modal.cycle')}
					<select bind:value={f.cycle}>
						<option value="monthly">{i18n.t('modal.cycleMonthly')}</option>
						<option value="yearly">{i18n.t('modal.cycleYearly')}</option>
						<option value="custom">{i18n.t('modal.cycleCustom')}</option>
						<option value="once">{i18n.t('modal.cycleOnce')}</option>
					</select>
				</label>
				{#if f.cycle === 'custom'}
					<label>
						{i18n.t('modal.everyDays')}
						<input bind:value={f.cycleDays} inputmode="numeric" />
					</label>
				{:else}
					{@render paymentSelect()}
				{/if}
			</div>

			<div class="grid2">
				<label>
					{i18n.t('modal.start')}
					<input type="date" bind:value={f.startDate} />
				</label>
				<label>
					{i18n.t('modal.due')}
					<input type="date" bind:value={f.dueDate} oninput={() => (f.dueTouched = true)} />
				</label>
			</div>

			{#if f.cycle === 'custom'}
				{@render paymentSelect()}
			{/if}

			<label>
				{i18n.t('modal.category')}
				<select bind:value={f.categoryId}>
					<option value="">{i18n.t('modal.noCategory')}</option>
					{#each categories as c (c.id)}
						<option value={String(c.id)}>{c.name}</option>
					{/each}
				</select>
			</label>

			{#if editing}
				<ReminderEditor subId={editing.id} />
			{/if}

			{#if error}<p class="err">{error}</p>{/if}

			<div class="foot">
				<button type="button" class="ghost" onclick={close}>{i18n.t('common.cancel')}</button>
				<button type="submit" class="primary" disabled={saving}>
					{saving
						? i18n.t('common.saving')
						: editing
							? i18n.t('common.save')
							: i18n.t('common.create')}
				</button>
			</div>
		</form>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 50;
		display: grid;
		place-items: center;
		padding: 1.5rem;
		background: rgba(2, 5, 12, 0.55);
		backdrop-filter: blur(4px);
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 0.7rem;
		width: 100%;
		max-width: 440px;
		padding: 1.5rem;
		border-radius: var(--radius-xl, 20px);
	}
	header {
		display: flex;
		align-items: center;
		gap: 0.65rem;
		margin-bottom: 0.25rem;
	}
	.chip {
		display: grid;
		place-items: center;
		width: 36px;
		height: 36px;
		border-radius: 10px;
		color: var(--cc);
		background: color-mix(in srgb, var(--cc) 16%, transparent);
		border: 1px solid color-mix(in srgb, var(--cc) 30%, transparent);
		flex: none;
	}
	h2 {
		flex: 1;
		margin: 0;
		font-size: 1.05rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.x {
		display: grid;
		place-items: center;
		width: 32px;
		height: 32px;
		border-radius: 9px;
		border: 1px solid var(--border);
		background: transparent;
		color: var(--text-2);
		cursor: pointer;
	}
	.x:hover {
		color: var(--text);
	}
	label {
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
		font-size: 0.78rem;
		color: var(--text-2);
	}
	.grid2 {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.7rem;
	}
	input,
	select {
		padding: 0.55rem 0.7rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text);
		font-size: 0.92rem;
	}
	input:focus-visible,
	select:focus-visible {
		outline: 2px solid var(--brand);
		outline-offset: 1px;
		border-color: transparent;
	}
	.err {
		margin: 0;
		color: var(--danger);
		font-size: 0.82rem;
	}
	/* Buttons split 50/50 full width: same size, hierarchy by color */
	.foot {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.6rem;
		margin-top: 0.5rem;
	}
	.ghost,
	.primary {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		height: 42px;
		border-radius: 11px;
		font-weight: 650;
		font-size: 0.92rem;
		cursor: pointer;
	}
	.ghost {
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text-2);
	}
	.ghost:hover {
		color: var(--text);
		border-color: var(--border-strong);
	}
	.primary {
		border: none;
		color: white;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
	}
	.primary:disabled {
		opacity: 0.7;
		cursor: default;
	}
</style>
