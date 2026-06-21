<script lang="ts">
	import { Check } from '@lucide/svelte';
	import { updateSettings, type User } from '$lib/api';
	import { i18n } from '$lib/i18n.svelte';
	import { busy } from './busy.svelte';

	// Identity is loaded once by the parent; we seed our editable copy from it.
	let { user }: { user: User | null } = $props();

	// Common zones (LATAM + a few) + UTC. The backend validates against IANA.
	const TIMEZONES = [
		'UTC',
		'America/Lima',
		'America/Bogota',
		'America/Mexico_City',
		'America/Argentina/Buenos_Aires',
		'America/Santiago',
		'America/Caracas',
		'America/Sao_Paulo',
		'America/New_York',
		'America/Los_Angeles',
		'Europe/Madrid'
	];
	const CURRENCIES = [
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

	let timezone = $state('UTC');
	let sendHour = $state(9);
	let defaultCurrency = $state('USD');
	let msg = $state<{ ok: boolean; text: string } | null>(null);
	const saver = busy();

	let seeded = false;
	$effect(() => {
		if (user && !seeded) {
			seeded = true;
			timezone = user.timezone;
			sendHour = user.send_hour;
			defaultCurrency = user.default_currency;
		}
	});

	function savePrefs() {
		saver.run(async () => {
			const res = await updateSettings({
				timezone,
				send_hour: sendHour,
				default_currency: defaultCurrency
			});
			msg = { ok: res.ok, text: res.ok ? i18n.t('set.savedOk') : i18n.t('set.saveErr') };
		});
	}
</script>

<div class="grid2">
	<label class="field">
		{i18n.t('set.timezone')}
		<select bind:value={timezone}>
			{#if !TIMEZONES.includes(timezone)}<option value={timezone}>{timezone}</option>{/if}
			{#each TIMEZONES as tz (tz)}<option value={tz}>{tz}</option>{/each}
		</select>
	</label>
	<label class="field">
		{i18n.t('set.sendHour')}
		<select bind:value={sendHour}>
			{#each Array(24) as _, h (h)}
				<option value={h}>{String(h).padStart(2, '0')}:00</option>
			{/each}
		</select>
	</label>
	<label class="field">
		{i18n.t('set.mainCurrency')}
		<select bind:value={defaultCurrency}>
			{#if !CURRENCIES.includes(defaultCurrency)}<option value={defaultCurrency}
					>{defaultCurrency}</option
				>{/if}
			{#each CURRENCIES as c (c)}<option value={c}>{c}</option>{/each}
		</select>
	</label>
</div>

<div class="actions">
	<button class="primary" onclick={savePrefs} disabled={saver.on}>
		<Check size={15} />
		{saver.on ? i18n.t('common.saving') : i18n.t('common.save')}
	</button>
	{#if msg}<span class:ok={msg.ok} class:err={!msg.ok} style="align-self:center">{msg.text}</span
		>{/if}
</div>

<style>
	.ok {
		color: var(--ok);
		font-size: 0.82rem;
	}
	.err {
		color: var(--danger);
		font-size: 0.82rem;
	}
</style>
