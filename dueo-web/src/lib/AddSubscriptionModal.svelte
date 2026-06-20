<script lang="ts">
	import { X, Plus, Sparkles, Search } from '@lucide/svelte';
	import {
		createSubscription,
		updateSubscription,
		getReminders,
		createReminder,
		deleteReminder,
		type Sub,
		type NewSub,
		type Category,
		type Reminder
	} from './api';
	import Icon from './Icon.svelte';
	import { searchIcons, brandsReady, DUEO_COLORS, resolveSubVisual } from './icons';
	import { ensureBrands } from './brandcat.svelte';
	import { i18n, reminderLabel } from './i18n.svelte';
	import { fade, scale } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';

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

	let name = $state('');
	let amount = $state('');
	let currency = $state('USD');
	let cycle = $state('monthly');
	let cycleDays = $state('30');
	let startDate = $state(today());
	let dueDate = $state('');
	let dueTouched = $state(false);
	let paymentMode = $state('manual');
	let categoryId = $state('');
	let notes = $state('');
	let error = $state('');
	let saving = $state(false);

	// Icono y color: null = automático (marca por nombre / tono por defecto).
	let icon = $state<string | null>(null);
	let color = $state<string | null>(null);
	let iconQuery = $state('');
	// Reactivo al catálogo perezoso: genéricos al instante; las marcas aparecen
	// cuando el usuario enfoca el buscador (carga bajo demanda).
	const filteredIcons = $derived(searchIcons(iconQuery));

	// Recordatorios PROPIOS de la sub (override de los globales, R11). Solo en edición.
	let subReminders = $state<Reminder[]>([]);
	let newReminderDays = $state('');
	const sortedReminders = $derived([...subReminders].sort((a, b) => a.days_before - b.days_before));

	// Lo que se mostrará (icono explícito → marca por nombre → genérico) + color.
	const preview = $derived(resolveSubVisual({ name, icon, color }));

	// Monedas ofrecidas (ISO 4217): mayores + LATAM. Si la sub en edición trae
	// una moneda fuera de la lista, la añadimos para no perderla.
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
		COMMON_CURRENCIES.includes(currency) || !currency
			? COMMON_CURRENCIES
			: [currency, ...COMMON_CURRENCIES]
	);

	// Autosugerir vencimiento desde inicio + ciclo, salvo que el usuario lo edite.
	$effect(() => {
		const s = startDate;
		const c = cycle;
		const n = parseInt(cycleDays) || 0;
		if (dueTouched || !s) return;
		const d = new Date(s + 'T00:00:00');
		if (c === 'monthly') d.setMonth(d.getMonth() + 1);
		else if (c === 'yearly') d.setFullYear(d.getFullYear() + 1);
		else if (c === 'custom') d.setDate(d.getDate() + n);
		else return; // once
		dueDate = d.toISOString().slice(0, 10);
	});

	// Preload ONCE on the open transition, not reactively: re-rendering the
	// `editing` object must not clobber edits already in progress.
	let lastOpened = $state(false);
	$effect(() => {
		if (open && !lastOpened) {
			lastOpened = true;
			if (editing) {
				name = editing.name;
				amount = (editing.amount_cents / 100).toString();
				currency = editing.currency;
				cycle = editing.cycle;
				cycleDays = String(editing.cycle_days ?? 30);
				startDate = editing.start_date;
				dueDate = editing.due_date;
				dueTouched = true;
				paymentMode = editing.payment_mode;
				categoryId = editing.category_id ? String(editing.category_id) : '';
				notes = editing.notes ?? '';
				icon = editing.icon ?? null;
				color = editing.color ?? null;
				loadSubReminders(editing.id);
			} else {
				currency = defaultCurrency; // new sub: seed with the user's main currency
			}
		} else if (!open && lastOpened) {
			lastOpened = false;
		}
	});

	async function loadSubReminders(subId: number) {
		subReminders = [];
		const res = await getReminders();
		// Discard a stale load if the edit target changed while we awaited.
		if (res.ok && editing?.id === subId) {
			const all: Reminder[] = await res.json();
			subReminders = all.filter((r) => r.subscription_id === subId);
		}
	}

	async function addSubReminder() {
		if (!editing) return;
		const n = parseInt(newReminderDays, 10);
		if (isNaN(n) || n < 0 || subReminders.some((r) => r.days_before === n)) return;
		const res = await createReminder({ subscription_id: editing.id, days_before: n });
		if (res.ok) {
			subReminders = [...subReminders, await res.json()];
			newReminderDays = '';
		}
	}

	async function removeSubReminder(r: Reminder) {
		const res = await deleteReminder(r.id);
		if (res.ok || res.status === 404) subReminders = subReminders.filter((x) => x.id !== r.id);
	}

	function reset() {
		name = '';
		amount = '';
		currency = 'USD';
		cycle = 'monthly';
		cycleDays = '30';
		startDate = today();
		dueDate = '';
		dueTouched = false;
		paymentMode = 'manual';
		categoryId = '';
		notes = '';
		error = '';
		icon = null;
		color = null;
		iconQuery = '';
		subReminders = [];
		newReminderDays = '';
	}

	function close() {
		reset();
		onclose?.();
	}

	async function submit(e: Event) {
		e.preventDefault();
		error = '';
		const cents = Math.round(parseFloat(amount.replace(',', '.')) * 100);
		if (!name.trim()) return (error = i18n.t('modal.errName'));
		if (isNaN(cents) || cents < 0) return (error = i18n.t('modal.errAmount'));
		if (!startDate || !dueDate) return (error = i18n.t('modal.errDates'));

		saving = true;
		try {
			const body: NewSub = {
				name: name.trim(),
				amount_cents: cents,
				currency,
				cycle,
				cycle_days: cycle === 'custom' ? parseInt(cycleDays) || null : null,
				start_date: startDate,
				due_date: dueDate,
				category_id: categoryId ? Number(categoryId) : null,
				payment_mode: paymentMode,
				notes: notes.trim() || null,
				icon,
				color
			};
			const res = editing
				? await updateSubscription(editing.id, body)
				: await createSubscription(body);
			if (!res.ok) return (error = editing ? i18n.t('modal.errSave') : i18n.t('modal.errCreate'));
			onsaved?.(await res.json());
			close();
		} catch {
			error = i18n.t('common.connError');
		} finally {
			saving = false;
		}
	}
</script>

<svelte:window onkeydown={(e) => open && e.key === 'Escape' && close()} />

{#if open}
	<!-- cierra solo si el clic cae en el backdrop, no dentro de la tarjeta (así el
	     form no necesita un onclick que dispara warnings de a11y). -->
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
				<h2>{name.trim() || i18n.t('modal.newSub')}</h2>
				<button type="button" class="x" onclick={close} aria-label={i18n.t('common.close')}
					><X size={18} /></button
				>
			</header>

			<label>
				{i18n.t('modal.name')}
				<input bind:value={name} placeholder={i18n.t('modal.namePlaceholder')} required />
			</label>

			<!-- Icono y color -->
			<div class="vis">
				<div class="vishead">
					<span class="rlabel">{i18n.t('modal.iconColor')}</span>
					<button
						type="button"
						class="auto"
						class:on={icon === null && color === null}
						onclick={() => {
							icon = null;
							color = null;
						}}
					>
						<Sparkles size={13} />
						{i18n.t('modal.auto')}
					</button>
				</div>

				<div class="searchrow">
					<Search size={14} />
					<input
						bind:value={iconQuery}
						onfocus={ensureBrands}
						placeholder={i18n.t('modal.iconSearch')}
					/>
					{#if iconQuery && !brandsReady()}<span
							class="loadingdot"
							title={i18n.t('modal.loadingBrands')}
						></span>{/if}
				</div>

				<div class="iconscroll">
					{#each filteredIcons as ic (ic.id)}
						<button
							type="button"
							class="iconbtn"
							class:sel={icon === ic.id}
							title={ic.label}
							aria-label={ic.label}
							onclick={() => (icon = ic.id)}
						>
							<Icon def={ic} size={18} />
						</button>
					{/each}
				</div>

				<div class="colorrow">
					{#each DUEO_COLORS as c (c)}
						<button
							type="button"
							class="dot"
							class:on={color === c}
							style="--d:{c}"
							aria-label={i18n.t('modal.color')}
							onclick={() => (color = c)}
						></button>
					{/each}
					<label class="custom" title={i18n.t('modal.customColor')}>
						<input type="color" oninput={(e) => (color = e.currentTarget.value)} />
						<span style="--d:{color ?? 'var(--brand)'}"></span>
					</label>
				</div>
			</div>

			<div class="grid2">
				<label>
					{i18n.t('modal.amount')}
					<input bind:value={amount} inputmode="decimal" placeholder="9.99" />
				</label>
				<label>
					{i18n.t('modal.currency')}
					<select bind:value={currency}>
						{#each currencyOptions as c (c)}
							<option value={c}>{c}</option>
						{/each}
					</select>
				</label>
			</div>

			<div class="grid2">
				<label>
					{i18n.t('modal.cycle')}
					<select bind:value={cycle}>
						<option value="monthly">{i18n.t('modal.cycleMonthly')}</option>
						<option value="yearly">{i18n.t('modal.cycleYearly')}</option>
						<option value="custom">{i18n.t('modal.cycleCustom')}</option>
						<option value="once">{i18n.t('modal.cycleOnce')}</option>
					</select>
				</label>
				{#if cycle === 'custom'}
					<label>
						{i18n.t('modal.everyDays')}
						<input bind:value={cycleDays} inputmode="numeric" />
					</label>
				{:else}
					<label>
						{i18n.t('modal.payment')}
						<select bind:value={paymentMode}>
							<option value="manual">{i18n.t('modal.paymentManual')}</option>
							<option value="auto">{i18n.t('modal.paymentAuto')}</option>
						</select>
					</label>
				{/if}
			</div>

			<div class="grid2">
				<label>
					{i18n.t('modal.start')}
					<input type="date" bind:value={startDate} />
				</label>
				<label>
					{i18n.t('modal.due')}
					<input type="date" bind:value={dueDate} oninput={() => (dueTouched = true)} />
				</label>
			</div>

			{#if cycle === 'custom'}
				<label>
					{i18n.t('modal.payment')}
					<select bind:value={paymentMode}>
						<option value="manual">{i18n.t('modal.paymentManual')}</option>
						<option value="auto">{i18n.t('modal.paymentAuto')}</option>
					</select>
				</label>
			{/if}

			<label>
				{i18n.t('modal.category')}
				<select bind:value={categoryId}>
					<option value="">{i18n.t('modal.noCategory')}</option>
					{#each categories as c (c.id)}
						<option value={String(c.id)}>{c.name}</option>
					{/each}
				</select>
			</label>

			{#if editing}
				<div class="reminders">
					<span class="rlabel">{i18n.t('modal.remTitle')}</span>
					<p class="rhint">{i18n.t('modal.remHint')}</p>
					<div class="rchips">
						{#each sortedReminders as r (r.id)}
							<span class="rchip">
								{reminderLabel(r.days_before)}
								<button
									type="button"
									onclick={() => removeSubReminder(r)}
									aria-label={i18n.t('common.delete')}
								>
									<X size={12} />
								</button>
							</span>
						{/each}
						{#if sortedReminders.length === 0}
							<span class="rmuted">{i18n.t('modal.remUseGlobal')}</span>
						{/if}
					</div>
					<div class="raddrow">
						<input
							bind:value={newReminderDays}
							inputmode="numeric"
							placeholder={i18n.t('modal.daysBefore')}
							onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addSubReminder())}
						/>
						<button type="button" class="radd" onclick={addSubReminder}
							><Plus size={14} /> {i18n.t('common.add')}</button
						>
					</div>
				</div>
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
	.reminders {
		display: flex;
		flex-direction: column;
		gap: 0.45rem;
		padding: 0.7rem 0.8rem;
		border: 1px solid var(--border);
		border-radius: 12px;
		background: var(--surface-2);
	}
	.rlabel {
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--text);
	}
	.rhint {
		margin: 0;
		font-size: 0.72rem;
		color: var(--text-muted);
	}
	.rchips {
		display: flex;
		flex-wrap: wrap;
		gap: 0.4rem;
		align-items: center;
	}
	.rchip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 5px 3px 9px;
		border-radius: 999px;
		font-size: 0.76rem;
		color: var(--text);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--brand) 28%, transparent);
	}
	.rchip button {
		display: grid;
		place-items: center;
		width: 16px;
		height: 16px;
		border: none;
		border-radius: 999px;
		background: color-mix(in srgb, var(--text) 10%, transparent);
		color: var(--text-2);
		cursor: pointer;
	}
	.rchip button:hover {
		color: var(--text);
		background: color-mix(in srgb, var(--danger) 22%, transparent);
	}
	.rmuted {
		font-size: 0.76rem;
		color: var(--text-muted);
	}
	.raddrow {
		display: flex;
		gap: 0.4rem;
	}
	.raddrow input {
		flex: 1;
	}
	.radd {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 0 0.7rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--surface);
		color: var(--text-2);
		font-size: 0.8rem;
		font-weight: 550;
		cursor: pointer;
	}
	.radd:hover {
		color: var(--text);
		border-color: var(--border-strong);
	}
	/* --- Icono y color --- */
	.vis {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		padding: 0.7rem 0.8rem;
		border: 1px solid var(--border);
		border-radius: 12px;
		background: var(--surface-2);
	}
	.vishead {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.auto {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 9px;
		border-radius: 999px;
		border: 1px solid var(--border);
		background: var(--surface);
		color: var(--text-2);
		font-size: 0.74rem;
		font-weight: 600;
		cursor: pointer;
	}
	.auto.on {
		color: white;
		border-color: transparent;
		background: linear-gradient(135deg, var(--brand), var(--brand-2));
	}
	.searchrow {
		display: flex;
		align-items: center;
		gap: 0.45rem;
		padding: 0 0.6rem;
		border: 1px solid var(--border);
		border-radius: 10px;
		background: var(--surface);
		color: var(--text-muted);
	}
	.searchrow input {
		border: none;
		background: transparent;
		padding: 0.45rem 0;
		flex: 1;
		color: var(--text);
		font-size: 0.86rem;
	}
	.searchrow input:focus-visible {
		outline: none;
	}
	.loadingdot {
		width: 12px;
		height: 12px;
		border-radius: 999px;
		border: 2px solid var(--border);
		border-top-color: var(--brand);
		animation: spin 0.7s linear infinite;
		flex: none;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.iconscroll {
		display: grid;
		grid-template-columns: repeat(8, 1fr);
		gap: 5px;
		max-height: 132px;
		overflow-y: auto;
		padding: 2px;
	}
	.iconbtn {
		display: grid;
		place-items: center;
		aspect-ratio: 1;
		border-radius: 9px;
		border: 1px solid var(--border);
		background: var(--surface);
		color: var(--text-2);
		cursor: pointer;
		transition:
			color 0.12s,
			border-color 0.12s,
			transform 0.12s;
	}
	.iconbtn:hover {
		color: var(--text);
		border-color: var(--border-strong);
		transform: translateY(-1px);
	}
	.iconbtn.sel {
		color: var(--brand);
		border-color: var(--brand);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
	}
	.colorrow {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 7px;
	}
	.dot {
		width: 22px;
		height: 22px;
		border-radius: 999px;
		border: 2px solid transparent;
		background: var(--d);
		cursor: pointer;
		padding: 0;
		transition: transform 0.12s;
	}
	.dot:hover {
		transform: scale(1.12);
	}
	.dot.on {
		border-color: var(--text);
	}
	.custom {
		position: relative;
		width: 22px;
		height: 22px;
		cursor: pointer;
	}
	.custom input {
		position: absolute;
		inset: 0;
		opacity: 0;
		cursor: pointer;
	}
	.custom span {
		display: block;
		width: 22px;
		height: 22px;
		border-radius: 999px;
		background: conic-gradient(from 0deg, #ef6b3d, #ef4da3, #a64def, #4d74ef, #ef6b3d);
		border: 2px solid var(--border-strong);
	}
	.err {
		margin: 0;
		color: var(--danger);
		font-size: 0.82rem;
	}
	/* botones a 50/50 de todo el ancho: mismo tamaño, jerarquía por color */
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
