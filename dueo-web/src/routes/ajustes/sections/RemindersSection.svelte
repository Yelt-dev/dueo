<script lang="ts">
	import { onMount } from 'svelte';
	import { Plus, X } from '@lucide/svelte';
	import { getReminders, createReminder, deleteReminder, type Reminder } from '$lib/api';
	import { i18n, reminderLabel } from '$lib/i18n.svelte';

	// Only the user's GLOBAL rules (per-subscription ones live in the edit modal).
	let reminders = $state<Reminder[]>([]);
	const globals = $derived(
		reminders
			.filter((r) => r.subscription_id === null)
			.sort((a, b) => a.days_before - b.days_before)
	);
	let newDays = $state('');
	let remErr = $state('');

	onMount(async () => {
		const r = await getReminders();
		if (r.ok && r.data) reminders = r.data;
	});

	async function addRule() {
		remErr = '';
		const n = parseInt(newDays, 10);
		if (isNaN(n) || n < 0) return (remErr = i18n.t('set.remErrInvalid'));
		if (globals.some((g) => g.days_before === n)) return (remErr = i18n.t('set.remErrExists'));
		const res = await createReminder({ subscription_id: null, days_before: n });
		if (res.ok && res.data) {
			reminders = [...reminders, res.data];
			newDays = '';
		} else {
			remErr = i18n.t('set.remErrAdd');
		}
	}

	async function removeRule(r: Reminder) {
		const res = await deleteReminder(r.id);
		if (res.ok || res.status === 404) reminders = reminders.filter((x) => x.id !== r.id);
	}
</script>

<div class="chips">
	{#each globals as g (g.id)}
		<span class="chip">
			{reminderLabel(g.days_before)}
			<button onclick={() => removeRule(g)} aria-label={i18n.t('common.delete')}
				><X size={13} /></button
			>
		</span>
	{/each}
	{#if globals.length === 0}
		<span class="muted">{i18n.t('set.remEmpty')}</span>
	{/if}
</div>

<form class="addrule" onsubmit={(e) => (e.preventDefault(), addRule())}>
	<input bind:value={newDays} inputmode="numeric" placeholder={i18n.t('set.remPlaceholder')} />
	<button type="submit" class="ghost"><Plus size={15} /> {i18n.t('common.add')}</button>
</form>
{#if remErr}<p class="err">{remErr}</p>{/if}

<style>
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		align-items: center;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 5px 6px 5px 11px;
		border-radius: 999px;
		font-size: 0.82rem;
		font-weight: 550;
		color: var(--text);
		background: color-mix(in srgb, var(--brand) 14%, transparent);
		border: 1px solid color-mix(in srgb, var(--brand) 30%, transparent);
	}
	.chip button {
		display: grid;
		place-items: center;
		width: 18px;
		height: 18px;
		border: none;
		border-radius: 999px;
		background: color-mix(in srgb, var(--text) 10%, transparent);
		color: var(--text-2);
		cursor: pointer;
	}
	.chip button:hover {
		color: var(--text);
		background: color-mix(in srgb, var(--danger) 22%, transparent);
	}
	.addrule {
		display: flex;
		gap: 0.5rem;
	}
	.addrule input {
		flex: 1;
	}
	.err {
		margin: 0;
		color: var(--danger);
		font-size: 0.82rem;
	}
</style>
