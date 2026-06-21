<script lang="ts">
	import { onMount } from 'svelte';
	import { Plus, X } from '@lucide/svelte';
	import { getReminders, createReminder, deleteReminder, type Reminder } from './api';
	import { i18n, reminderLabel } from './i18n.svelte';

	// Per-subscription reminder rules (override the user's global rules, R11).
	let { subId }: { subId: number } = $props();

	let subReminders = $state<Reminder[]>([]);
	let newReminderDays = $state('');
	const sorted = $derived([...subReminders].sort((a, b) => a.days_before - b.days_before));

	onMount(async () => {
		const res = await getReminders();
		if (res.ok && res.data) subReminders = res.data.filter((r) => r.subscription_id === subId);
	});

	async function add() {
		const n = parseInt(newReminderDays, 10);
		if (isNaN(n) || n < 0 || subReminders.some((r) => r.days_before === n)) return;
		const res = await createReminder({ subscription_id: subId, days_before: n });
		if (res.ok && res.data) {
			subReminders = [...subReminders, res.data];
			newReminderDays = '';
		}
	}

	async function remove(r: Reminder) {
		const res = await deleteReminder(r.id);
		if (res.ok || res.status === 404) subReminders = subReminders.filter((x) => x.id !== r.id);
	}
</script>

<div class="reminders">
	<span class="rlabel">{i18n.t('modal.remTitle')}</span>
	<p class="rhint">{i18n.t('modal.remHint')}</p>
	<div class="rchips">
		{#each sorted as r (r.id)}
			<span class="rchip">
				{reminderLabel(r.days_before)}
				<button type="button" onclick={() => remove(r)} aria-label={i18n.t('common.delete')}>
					<X size={12} />
				</button>
			</span>
		{/each}
		{#if sorted.length === 0}
			<span class="rmuted">{i18n.t('modal.remUseGlobal')}</span>
		{/if}
	</div>
	<div class="raddrow">
		<input
			bind:value={newReminderDays}
			inputmode="numeric"
			placeholder={i18n.t('modal.daysBefore')}
			onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), add())}
		/>
		<button type="button" class="radd" onclick={add}
			><Plus size={14} /> {i18n.t('common.add')}</button
		>
	</div>
</div>

<style>
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
		padding: 0.55rem 0.7rem;
		border-radius: 10px;
		border: 1px solid var(--border);
		background: var(--surface-2);
		color: var(--text);
		font-size: 0.92rem;
	}
	.raddrow input:focus-visible {
		outline: 2px solid var(--brand);
		outline-offset: 1px;
		border-color: transparent;
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
</style>
