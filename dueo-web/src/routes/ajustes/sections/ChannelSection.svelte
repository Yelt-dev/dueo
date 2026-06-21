<script lang="ts">
	import { Check, TriangleAlert } from '@lucide/svelte';
	import { onMount } from 'svelte';
	import { i18n } from '$lib/i18n.svelte';
	import { busy } from './busy.svelte';
	import FormMessage from './FormMessage.svelte';
	import type { ChannelConfig } from './channel';

	let { config }: { config: ChannelConfig } = $props();
	const TestIcon = $derived(config.icon);

	let loaded = $state(false);
	let ready = $state(false); // server has the token/SMTP configured
	let value = $state('');
	let enabled = $state(false);
	let msg = $state<{ ok: boolean; text: string } | null>(null);
	const saver = busy();
	const tester = busy();

	onMount(async () => {
		const s = await config.load();
		if (s) {
			ready = s.ready;
			value = s.value;
			enabled = s.enabled;
		}
		loaded = true;
	});

	function save() {
		msg = null;
		saver.run(async () => {
			const res = await config.save(value.trim(), enabled);
			if (res.ok) value = value.trim();
			else msg = { ok: false, text: i18n.t('set.saveErr') };
		});
	}

	function sendTest() {
		msg = null;
		tester.run(async () => {
			try {
				const res = await config.test();
				msg = res.ok
					? { ok: true, text: i18n.t(config.testOkKey) }
					: { ok: false, text: i18n.t(config.testErrKey) };
			} catch {
				msg = { ok: false, text: i18n.t('common.connError') };
			}
		});
	}
</script>

{#if loaded && !ready}
	<div class="warn">
		<TriangleAlert size={15} />
		{i18n.t(config.warnPreKey)}<code>{config.warnCode}</code>).
	</div>
{/if}

<label class="field">
	{i18n.t(config.fieldKey)}
	<input bind:value type={config.inputType} placeholder={config.placeholder} />
	<span class="hint">{i18n.t(config.hintKey)}</span>
</label>

<label class="check">
	<input type="checkbox" bind:checked={enabled} />
	{i18n.t(config.enableKey)}
</label>

<div class="actions">
	<button class="primary" onclick={save} disabled={saver.on}>
		<Check size={15} />
		{saver.on ? i18n.t('common.saving') : i18n.t('common.save')}
	</button>
	<button class="ghost" onclick={sendTest} disabled={tester.on || !ready || !value.trim()}>
		<TestIcon size={15} />
		{tester.on ? i18n.t('set.tgTesting') : i18n.t('set.tgTest')}
	</button>
</div>

<FormMessage {msg} />
