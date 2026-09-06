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
	import { fade, scale, slide } from 'svelte/transition';
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
			endDate: '',
			openEnded: true,
			startTouched: false,
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
			endDate: s.end_date ?? '',
			openEnded: !s.end_date,
			startTouched: true,
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

	// Move a date one cycle forward (dir 1) or back (dir -1). Months clamp to the
	// last day, matching the backend's chrono `checked_add_months`. Returns null
	// for 'once' (no recurrence) or an unusable input.
	function shiftCycle(iso: string, cycle: string, dir: 1 | -1, days: string): string | null {
		if (!iso) return null;
		const d = new Date(iso + 'T00:00:00');
		if (isNaN(d.getTime())) return null;
		if (cycle === 'custom') {
			const n = parseInt(days) || 0;
			if (n < 1) return null;
			d.setDate(d.getDate() + dir * n);
		} else if (cycle === 'monthly' || cycle === 'yearly') {
			const step = cycle === 'yearly' ? 12 : 1;
			const day = d.getDate();
			d.setDate(1);
			d.setMonth(d.getMonth() + dir * step);
			const lastDay = new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate();
			d.setDate(Math.min(day, lastDay));
		} else {
			return null; // once
		}
		return d.toISOString().slice(0, 10);
	}

	// The two dates derive from each other through the cycle, so the user fills in
	// whichever one they actually know. Only the untouched side is ever written,
	// which is also why these can't ping-pong.
	$effect(() => {
		if (f.dueTouched || !f.startDate) return;
		const next = shiftCycle(f.startDate, f.cycle, 1, f.cycleDays);
		if (next) f.dueDate = next;
	});
	$effect(() => {
		if (!f.dueTouched || f.startTouched || !f.dueDate) return;
		const prev = shiftCycle(f.dueDate, f.cycle, -1, f.cycleDays);
		if (prev) f.startDate = prev;
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
		// A recurring service has no end date: whichever date is missing is derived
		// from the cycle. Only 'once' needs a real due date typed in.
		const startDate = f.startDate || shiftCycle(f.dueDate, f.cycle, -1, f.cycleDays) || '';
		const dueDate = f.dueDate || shiftCycle(startDate, f.cycle, 1, f.cycleDays) || '';
		if (!dueDate)
			return (error = i18n.t(f.cycle === 'once' ? 'modal.errDueOnce' : 'modal.errDates'));
		if (!startDate) return (error = i18n.t('modal.errDates'));
		if (dueDate <= startDate) return (error = i18n.t('modal.errDateOrder')); // R4
		// Open-ended (the default) means no termination date at all; 'once' has no
		// renewal to stop, so its due date already is the end.
		const endDate = f.openEnded || f.cycle === 'once' ? null : f.endDate;
		if (endDate !== null && !endDate) return (error = i18n.t('modal.errEndMissing'));
		if (endDate && endDate <= startDate) return (error = i18n.t('modal.errEndOrder'));

		saving = true;
		try {
			const body: NewSub = {
				name: f.name.trim(),
				amount_cents: cents,
				currency: f.currency,
				cycle: f.cycle,
				cycle_days: f.cycle === 'custom' ? parseInt(f.cycleDays) || null : null,
				start_date: startDate,
				due_date: dueDate,
				end_date: endDate,
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

			<div class="body">
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
							<option value="quarterly">{i18n.t('modal.cycleQuarterly')}</option>
							<option value="semiannual">{i18n.t('modal.cycleSemiannual')}</option>
							<option value="yearly">{i18n.t('modal.cycleYearly')}</option>
							<option value="biennial">{i18n.t('modal.cycleBiennial')}</option>
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
						<input type="date" bind:value={f.startDate} oninput={() => (f.startTouched = true)} />
					</label>
					<label>
						{f.cycle === 'once' ? i18n.t('modal.due') : i18n.t('modal.nextCharge')}
						<input
							type="date"
							bind:value={f.dueDate}
							min={f.startDate || undefined}
							oninput={() => (f.dueTouched = true)}
						/>
					</label>
				</div>

				{#if f.cycle !== 'once'}
					<div class="ends">
						<label class="check">
							<input type="checkbox" bind:checked={f.openEnded} />
							<span>{i18n.t('modal.openEnded')}</span>
						</label>

						{#if f.openEnded}
							<p class="hint" transition:slide={{ duration: 160, easing: cubicOut }}>
								{f.paymentMode === 'auto'
									? i18n.t('modal.datesHintAuto')
									: i18n.t('modal.datesHintManual')}
							</p>
						{:else}
							<div class="endfield" transition:slide={{ duration: 160, easing: cubicOut }}>
								<label>
									{i18n.t('modal.endDate')}
									<input type="date" bind:value={f.endDate} min={f.startDate || undefined} />
								</label>
								<p class="hint">{i18n.t('modal.datesHintEnds')}</p>
							</div>
						{/if}
					</div>
				{:else}
					<p class="hint">{i18n.t('modal.onceHint')}</p>
				{/if}

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

				<label>
					{i18n.t('modal.notes')}
					<textarea bind:value={f.notes} rows="2" placeholder={i18n.t('modal.notesPlaceholder')}
					></textarea>
				</label>

				{#if editing}
					<ReminderEditor subId={editing.id} />
				{/if}
			</div>

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
		overflow-y: auto;
		background: rgba(2, 5, 12, 0.55);
		backdrop-filter: blur(4px);
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 0.7rem;
		width: 100%;
		max-width: 440px;
		max-height: calc(100vh - 3rem);
		max-height: calc(100dvh - 3rem);
		padding: 1.5rem;
		border-radius: var(--radius-xl, 20px);
	}
	/* The card never outgrows the viewport: only the fields scroll, while the
	   header and the actions stay put. */
	.body {
		display: flex;
		flex-direction: column;
		gap: 0.7rem;
		min-height: 0;
		overflow-y: auto;
		scrollbar-width: thin;
		/* bleed sideways so focus rings aren't clipped by the scroll container */
		margin: 0 -0.5rem;
		padding: 0.15rem 0.5rem;
	}
	header {
		display: flex;
		flex: none;
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
	select,
	textarea {
		padding: 0.55rem 0.7rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background-color: var(--surface-2);
		color: var(--text);
		font-size: 0.92rem;
	}
	select {
		padding-right: 2.1rem; /* room for the chevron (see app.css) */
	}
	textarea {
		font-family: inherit;
		resize: vertical;
	}
	input:focus-visible,
	select:focus-visible,
	textarea:focus-visible {
		outline: 2px solid var(--brand);
		outline-offset: 1px;
		border-color: transparent;
	}
	/* Termination block: the checkbox owns the whole "does this end?" question,
	   so a date only exists when the answer is yes. */
	.ends,
	.endfield {
		display: flex;
		flex-direction: column;
		gap: 0.45rem;
	}
	.check {
		flex-direction: row;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.82rem;
		color: var(--text);
		cursor: pointer;
	}
	.check input {
		width: 16px;
		height: 16px;
		padding: 0;
		accent-color: var(--brand);
		cursor: pointer;
	}
	.hint {
		margin: -0.15rem 0 0;
		font-size: 0.74rem;
		line-height: 1.35;
		color: var(--text-2);
	}
	.err {
		margin: 0;
		color: var(--danger);
		font-size: 0.82rem;
	}
	/* Buttons split 50/50 full width: same size, hierarchy by color */
	.foot {
		display: grid;
		flex: none;
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
